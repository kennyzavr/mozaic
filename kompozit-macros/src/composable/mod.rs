use proc_macro2::Span;
use syn::spanned::Spanned;

mod id;
mod patcher;
mod scope;
mod selection;

const NAMESPACE: &str = "N_";
const COMPOSE_KEYWORD: &str = "compose";

trait ComposeNode {
    fn id(&self) -> id::Id<impl AsRef<str>>;

    fn prologue(&self) -> proc_macro2::TokenStream;

    fn epilogue(&self) -> proc_macro2::TokenStream;
}

struct RootScope {}

fn root_scope_id_name() -> &'static str {
    "root scope"
}

pub(crate) fn transform(move_env: bool, mut expr: syn::Expr) -> syn::Block {
    let root_scope = RootScope {};
    let root_scope_gv = root_scope.id().gv();

    let mut patcher = patcher::Patcher {
        scope: scope::Scope::new(root_scope),
    };

    syn::visit_mut::VisitMut::visit_expr_mut(&mut patcher, &mut expr);
    let prologue = patcher.scope.prologue();
    let epilogue = patcher.scope.epilogue();

    let move_token = move_env.then_some(syn::Token![move](Span::mixed_site()));

    let expr_value_gv = id::Id {
        name: "expr value",
        span: expr.span(),
    }
    .gv();
    let inner = quote::quote_spanned! { Span::mixed_site() =>
        #prologue
        let #expr_value_gv = (move || #expr)();
        #epilogue
        #expr_value_gv
    };
    let inner = quote::quote_spanned! { expr.span() =>
        {
            #inner
        }
    };
    syn::parse_quote_spanned! { Span::mixed_site() =>
        {
            let recomp = ::kompozit::from_fn(#move_token |#[allow(non_snake_case)] #root_scope_gv: &mut _| #inner);

            {
                #[allow(unused_imports)]
                use ::kompozit::private::{Caster, CastNever, FallbackCastSecondary};
                let caster = &&Caster::new(&recomp);
                caster.cast(recomp)
            }
        }
    }
}

impl ComposeNode for RootScope {
    fn id(&self) -> id::Id<impl AsRef<str>> {
        id::Id {
            name: root_scope_id_name(),
            span: Span::mixed_site(),
        }
    }

    fn prologue(&self) -> proc_macro2::TokenStream {
        quote::quote! {}
    }

    fn epilogue(&self) -> proc_macro2::TokenStream {
        quote::quote! {}
    }
}
