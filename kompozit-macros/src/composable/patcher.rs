use proc_macro_error::emit_error;
use proc_macro2::Span;
use syn::{spanned::Spanned, visit::Visit, visit_mut::VisitMut};

use super::{COMPOSE_KEYWORD, ComposeNode, scope::Scope, selection::Selection};
use crate::composable::id::Id;

pub(super) struct Patcher<Parent> {
    pub scope: Scope<Parent>,
}

struct NotAllowedBindingsChecker<'expr> {
    expr: &'expr syn::Expr,
    cause: NotAllowedBindingCause,
}

enum NotAllowedBindingCause {
    Cycle,
    Closure,
    AsyncBlock,
    TryBlock,
}

impl<Parent: ComposeNode> Patcher<Parent> {
    fn patch_block_mut(&mut self, block: &mut syn::Block) {
        let mut unhandled_stmts = block.stmts.drain(..).collect::<Vec<_>>().into_iter();

        while let Some(stmt) = unhandled_stmts.next() {
            match stmt {
                syn::Stmt::Local(local) => {
                    block
                        .stmts
                        .extend(self.patch_local(local, &mut unhandled_stmts));
                }
                mut stmt => {
                    self.visit_stmt_mut(&mut stmt);
                    block.stmts.push(stmt);
                }
            }
        }
    }

    /// TODO: test bindings in a 'local' statements
    fn patch_local(
        &mut self,
        mut local: syn::Local,
        other_stmts: &mut impl Iterator<Item = syn::Stmt>,
    ) -> Vec<syn::Stmt> {
        self.visit_attributes_mut(&mut local.attrs);
        self.visit_pat_mut(&mut local.pat);

        let Some(init) = &mut local.init else {
            return vec![syn::Stmt::Local(local)];
        };

        self.visit_expr_mut(init.expr.as_mut());
        let Some((_, diverge_expr)) = init.diverge.as_mut() else {
            return vec![syn::Stmt::Local(local)];
        };

        let mut selection = {
            let scope_item = self.scope.add_item(Span::call_site());
            Selection::new(scope_item)
        };

        {
            let mut patcher = Patcher {
                scope: {
                    let variant = selection.add_variant(Span::call_site());
                    Scope::new(variant)
                },
            };

            patcher.visit_expr_mut(diverge_expr.as_mut());
            let prologue = patcher.scope.prologue();
            let epilogue = patcher.scope.epilogue();

            *diverge_expr.as_mut() = syn::parse_quote_spanned! { proc_macro2::Span::mixed_site() =>
                {
                    #prologue
                    #diverge_expr
                    #epilogue
                }
            };
        }

        let coverage_stmts = {
            let mut patcher = Patcher {
                scope: {
                    let variant = selection.add_variant(Span::call_site());
                    Scope::new(variant)
                },
            };

            let mut stmts: Vec<_> = other_stmts.collect();
            for stmt in stmts.iter_mut().skip(1) {
                patcher.visit_stmt_mut(stmt);
            }

            let prologue = patcher.scope.prologue();
            let epilogue = patcher.scope.epilogue();
            quote::quote! {
                #prologue
                #(#stmts)*
                #epilogue
            }
        };

        let prologue = selection.prologue();
        let epilogue = selection.epilogue();
        let compilation: syn::Block = syn::parse_quote_spanned! { proc_macro2::Span::mixed_site() =>
            {
                #prologue
                #local
                #coverage_stmts
                #epilogue
            }
        };

        compilation.stmts
    }

    fn patch_expr_if(&mut self, mut expr_if: syn::ExprIf) -> syn::Expr {
        self.visit_attributes_mut(&mut expr_if.attrs);
        self.visit_expr_mut(expr_if.cond.as_mut());

        let mut selection = {
            let scope_item = self.scope.add_item(expr_if.span());
            Selection::new(scope_item)
        };

        let then_reached_gv = Id {
            name: "then reached",
            span: Span::mixed_site(),
        }
        .gv();

        {
            let block = &mut expr_if.then_branch;
            let mut patcher = Patcher {
                scope: {
                    let variant = selection.add_variant(block.span());
                    Scope::new(variant)
                },
            };

            patcher.visit_block_mut(block);
            let prologue = patcher.scope.prologue();
            let epilogue = patcher.scope.epilogue();

            let expr_value_gv = Id {
                name: "expr value",
                span: block.span(),
            }
            .gv();
            let epilogue = quote::quote_spanned! { Span::mixed_site() =>
                #epilogue
                #expr_value_gv
            };
            let epilogue = quote::quote_spanned! { block.span() =>
                {
                    #epilogue
                }
            };
            let epilogue = quote::quote_spanned! { Span::mixed_site() =>
                    #prologue
                    #then_reached_gv = true;
                    let #expr_value_gv =#block;
                    #[allow(unreachable_code)]
                    #epilogue
            };
            *block = syn::parse_quote_spanned! { block.span() =>
                {
                    #epilogue
                }
            };
        }

        let state_reset = if let Some((_, block)) = expr_if.else_branch.as_mut() {
            let block = block.as_mut();
            let mut patcher = Patcher {
                scope: {
                    let variant = selection.add_variant(block.span());
                    Scope::new(variant)
                },
            };

            patcher.visit_expr_mut(block);
            let prologue = patcher.scope.prologue();
            let epilogue = patcher.scope.epilogue();

            let expr_value_gv = Id {
                name: "expr value",
                span: block.span(),
            }
            .gv();
            let epilogue = quote::quote_spanned! { Span::mixed_site() =>
                #epilogue
                #expr_value_gv
            };
            let epilogue = quote::quote_spanned! { block.span() =>
                {
                    #epilogue
                }
            };
            let epilogue = quote::quote_spanned! { Span::mixed_site() =>
                    #then_reached_gv = false;
                    #prologue
                    let #expr_value_gv = #block;
                    #[allow(unreachable_code)]
                    #epilogue
            };
            *block = syn::parse_quote_spanned! { block.span() =>
                {
                    #epilogue
                }
            };

            None
        } else {
            let patcher = Patcher {
                scope: {
                    let variant = selection.add_variant(Span::mixed_site());
                    Scope::new(variant)
                },
            };

            let prologue = patcher.scope.prologue();
            let epilogue = patcher.scope.epilogue();

            Some(quote::quote_spanned! { Span::mixed_site() =>
                if #then_reached_gv == false {
                    #prologue
                    #epilogue
                }
            })
        };

        let prologue = selection.prologue();
        let epilogue = selection.epilogue();
        let expr_value_gv = Id {
            name: "expr value",
            span: expr_if.span(),
        }
        .gv();
        let epilogue = quote::quote_spanned! { Span::mixed_site() =>
            #epilogue
            #expr_value_gv
        };
        let epilogue = quote::quote_spanned! { expr_if.span() =>
            {
                #epilogue
            }
        };
        syn::parse_quote_spanned! { Span::mixed_site() =>
            {
                #[allow(unused_mut)]
                #[allow(non_snake_case)]
                let mut #then_reached_gv = false;
                #prologue
                let #expr_value_gv = #expr_if;
                #state_reset
                #[allow(unreachable_code)]
                #epilogue
            }
        }
    }

    fn patch_expr_match(&mut self, mut expr_match: syn::ExprMatch) -> syn::Expr {
        self.visit_attributes_mut(&mut expr_match.attrs);
        self.visit_expr_mut(expr_match.expr.as_mut());

        let mut selection = {
            let scope_item = self.scope.add_item(expr_match.span());
            Selection::new(scope_item)
        };

        for arm in expr_match.arms.iter_mut() {
            let span = arm.span();
            let body = arm.body.as_mut();

            self.visit_attributes_mut(&mut arm.attrs);
            self.visit_pat_mut(&mut arm.pat);

            if let Some((_, guard)) = arm.guard.as_mut() {
                self.visit_expr_mut(guard);
            }

            let mut patcher = Patcher {
                scope: {
                    let variant = selection.add_variant(span);
                    Scope::new(variant)
                },
            };

            patcher.visit_expr_mut(body);
            let prologue = patcher.scope.prologue();
            let epilogue = patcher.scope.epilogue();
            let expr_value_gv = Id {
                name: "expr value",
                span,
            }
            .gv();
            let epilogue = quote::quote_spanned! { Span::mixed_site() =>
                #epilogue
                #expr_value_gv
            };
            let epilogue = quote::quote_spanned! { span =>
                {
                    #epilogue
                }
            };
            *body = syn::parse_quote_spanned! { proc_macro2::Span::mixed_site() =>
                {
                    #prologue
                    let expr_value_gv = #body;
                    #[allow(unreachable_code)]
                    #epilogue
                }
            };
        }

        let prologue = selection.prologue();
        let epilogue = selection.epilogue();
        let expr_value_gv = Id {
            name: "expr value",
            span: expr_match.span(),
        }
        .gv();
        let epilogue = quote::quote_spanned! { Span::mixed_site() =>
            #epilogue
            #expr_value_gv
        };
        let epilogue = quote::quote_spanned! { expr_match.span() =>
            {
                #epilogue
            }
        };
        syn::parse_quote_spanned! { proc_macro2::Span::mixed_site() =>
            {
                #prologue
                let #expr_value_gv = #expr_match;
                #[allow(unreachable_code)]
                #epilogue
            }
        }
    }

    fn patch_expr_field(&mut self, mut expr_field: syn::ExprField) -> syn::Expr {
        let syn::Member::Named(member) = &mut expr_field.member else {
            syn::visit_mut::visit_expr_field_mut(self, &mut expr_field);
            return expr_field.into();
        };

        if member.to_string() == format!("r#{COMPOSE_KEYWORD}") {
            *member = quote::format_ident!("{COMPOSE_KEYWORD}");
            syn::visit_mut::visit_expr_field_mut(self, &mut expr_field);
            return expr_field.into();
        }

        if member.to_string() != COMPOSE_KEYWORD {
            syn::visit_mut::visit_expr_field_mut(self, &mut expr_field);
            return expr_field.into();
        }

        let syn::ExprField {
            mut attrs,
            mut base,
            member,
            ..
        } = expr_field;

        self.visit_attributes_mut(&mut attrs);
        self.visit_expr_mut(&mut base);

        let scope_item = self.scope.add_item(member.span());
        let scope_item_gv = {
            let id = scope_item.id();
            id.gv()
        };
        let prologue = scope_item.prologue();
        let epilogue = scope_item.epilogue();

        let recomp_source_v = quote::format_ident!("recomp_source", span = member.span());
        let recomp_v = quote::format_ident!("recomp", span = member.span());

        let recomp_check = quote::quote_spanned! { member.span() =>
            ::kompozit::private::check_to_recomp(&#recomp_source_v);
        };

        syn::parse_quote_spanned! { proc_macro2::Span::mixed_site() =>
            {
                #prologue

                #[allow(non_snake_case)]
                let #scope_item_gv: &mut ::core::option::Option<_> = #scope_item_gv;
                let #recomp_source_v = #base;
                #recomp_check
                let #recomp_v = {
                    #[allow(unused_imports)]
                    use ::kompozit::private::{Caster, CastStub, FallbackCastPrimary};
                    let caster = &&Caster::new(&#recomp_source_v);
                    caster.cast(#recomp_source_v)
                };
                let recomposition = ::kompozit::Recomposition::apply(#recomp_v, #scope_item_gv.get_or_insert_with(|| ::kompozit::Composition::init()));

                #epilogue
                recomposition
            }
        }
    }
}

impl<Parent: ComposeNode> syn::visit_mut::VisitMut for Patcher<Parent> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        self.patch_block_mut(block);
    }

    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        *expr = match std::mem::replace(expr, syn::Expr::PLACEHOLDER) {
            syn::Expr::If(expr_if) => self.patch_expr_if(expr_if),
            syn::Expr::Match(expr_match) => self.patch_expr_match(expr_match),
            syn::Expr::Field(expr_field) => self.patch_expr_field(expr_field),
            expr @ syn::Expr::Closure(_) => {
                NotAllowedBindingsChecker {
                    expr: &expr,
                    cause: NotAllowedBindingCause::Closure,
                }
                .visit_expr(&expr);
                expr
            }
            expr @ syn::Expr::Loop(_)
            | expr @ syn::Expr::ForLoop(_)
            | expr @ syn::Expr::While(_) => {
                NotAllowedBindingsChecker {
                    expr: &expr,
                    cause: NotAllowedBindingCause::Cycle,
                }
                .visit_expr(&expr);
                expr
            }
            expr @ syn::Expr::Async(_) => {
                NotAllowedBindingsChecker {
                    expr: &expr,
                    cause: NotAllowedBindingCause::AsyncBlock,
                }
                .visit_expr(&expr);
                expr
            }
            expr @ syn::Expr::TryBlock(_) => {
                NotAllowedBindingsChecker {
                    expr: &expr,
                    cause: NotAllowedBindingCause::TryBlock,
                }
                .visit_expr(&expr);
                expr
            }
            mut expr => {
                syn::visit_mut::visit_expr_mut(self, &mut expr);
                expr
            }
        }
    }
}

impl<'expr> syn::visit::Visit<'expr> for NotAllowedBindingsChecker<'expr> {
    fn visit_expr_field(&mut self, field_expr: &syn::ExprField) {
        let syn::Member::Named(field_ident) = &field_expr.member else {
            return;
        };

        if field_ident.to_string() != COMPOSE_KEYWORD {
            return;
        }

        if field_ident.to_string() == format!("r#{COMPOSE_KEYWORD}") {
            return;
        }

        match self.cause {
            NotAllowedBindingCause::Cycle => {
                emit_error!(
                    self.expr,
                    "'{}' bindings are not allowed in cycles",
                    COMPOSE_KEYWORD
                );
            }
            NotAllowedBindingCause::Closure => {
                emit_error!(
                    self.expr,
                    "'{}' bindings are not allowed in closures",
                    COMPOSE_KEYWORD
                );
            }
            NotAllowedBindingCause::AsyncBlock => {
                emit_error!(
                    self.expr,
                    "'{}' bindings are not allowed in async blocks",
                    COMPOSE_KEYWORD
                );
            }
            NotAllowedBindingCause::TryBlock => {
                emit_error!(
                    self.expr,
                    "'{}' bindings are not allowed in try blocks",
                    COMPOSE_KEYWORD
                );
            }
        }
    }
}
