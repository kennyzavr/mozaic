use proc_macro2::Span;

use crate::ComposeNode;
use crate::id::EntityId;

pub struct Scope<Parent> {
    parent: Parent,
    span: Span,
    items: Vec<Span>,
}

pub struct ScopeItem {
    index: usize,
    span: Span,
    scope_span: Span,
}

fn scope_id_name() -> &'static str {
    "scope"
}

fn item_id_name(index: usize) -> String {
    format!("scope item {index}")
}

fn item_composer_id_name(index: usize) -> String {
format!("scope item {} composer", index)
}

impl<Parent: ComposeNode> Scope<Parent> {
    pub fn new(parent: Parent, span: Span) -> Self {
        Self {
            parent,
            span,
            items: vec![],
        }
    }

    pub fn add_item(&mut self, span: Span) -> ScopeItem {
        let index = self.items.len();
        self.items.push(span);
        ScopeItem {
            index,
            span,
            scope_span: self.span,
        }
    }

    pub fn prologue(&self) -> proc_macro2::TokenStream {
        let (scope_gt, scope_gv, scope_lv) = {
            let id = EntityId {
                name: scope_id_name(),
                span: Span::mixed_site(),
            };
            (id.gt(), id.gv(), id.lv())
        };

        let (item_lt, item_lv): (Vec<_>, Vec<_>) = self
            .items
            .iter()
            .enumerate()
            .map(|(index, span)| {
                let id = EntityId {
                    name: item_id_name(index),
                    span: Span::mixed_site(),
                };
                (id.lt(), id.lv())
            })
            .unzip();

        let item_infer = vec![quote::quote! { _ }; self.items.len()];

        let viewer_gt = EntityId {
            name: "scope viewer",
            span: Span::mixed_site(),
        }
        .gt();

        let viewer_state_gt = EntityId {
            name: "scope viewer state",
            span: Span::mixed_site(),
        }
        .gt();

        let begin_state_lt = quote::format_ident!("Begin", span = Span::mixed_site());
        let end_state_lt = quote::format_ident!("End", span = Span::mixed_site());

        let state_lt: Vec<_> = std::iter::once(&begin_state_lt)
            .chain(item_lt.iter())
            .chain(std::iter::once(&end_state_lt))
            .collect();
        let next_state_lt: Vec<_> = state_lt
            .iter()
            .copied()
            .skip(1)
            .chain(std::iter::once(&end_state_lt))
            .collect();
        let prev_state_lt: Vec<_> = state_lt
            .iter()
            .copied()
            .rev()
            .skip(1)
            .chain(std::iter::once(&begin_state_lt))
            .rev()
            .collect();

        let parent_gv = self.parent.id().gv();
        let parent_prologue = self.parent.prologue();

        let item = self.items.iter().enumerate()
            .map(|(index, span)| {
                let id = EntityId {
                    name: item_id_name(index),
                    span: Span::mixed_site(),
                };
                let current_item_gt = id.gt();
                let current_item_lt = id.lt();
                let current_item_lv = id.lv();

        quote::quote_spanned! { Span::mixed_site() =>
                    #[allow(non_camel_case_types)]
                    struct #current_item_gt<'s, 'ss, Unit: ?Sized, #(#item_lt: ::kompozit::Composition),*> {
                        #scope_lv: &'s mut &'ss mut #scope_gt<Unit, #(#item_lt),*>,
                    }

                    impl<'s, 'ss, Unit, #(#item_lt),*> ::core::convert::From<&'s mut &'ss mut #scope_gt<Unit, #(#item_lt),*>>
                        for #current_item_gt<'s, 'ss, Unit, #(#item_lt),*>
                        where
                            Unit: ?Sized,
                            #(#item_lt: ::kompozit::Composition),*
                    {
                        fn from(#scope_lv: &'s mut &'ss mut #scope_gt<Unit, #(#item_lt),*>) -> Self {
                            Self {
                                #scope_lv,
                            }
                        }
                    }

                    impl<'s, 'ss, Unit, #(#item_lt),*> ::kompozit::private::Slot
                        for #current_item_gt<'s, 'ss, Unit, #(#item_lt),*> 
                        where
                            Unit: ?Sized,
                            #(#item_lt: ::kompozit::Composition),*
                    {
                        type Source = &'s mut &'ss mut #scope_gt<Unit, #(#item_lt),*>;
                        type Target = #current_item_lt;

                        fn get(&mut self) -> &mut Self::Target {
                            &mut self.#scope_lv.#current_item_lv
                        }
                    }
                }
            });

        quote::quote_spanned! { Span::mixed_site() =>
            #[allow(non_camel_case_types)]
            struct #scope_gt<Unit: ?Sized, #(#item_lt: ::kompozit::Composition),*> {
                _unit: ::core::marker::PhantomData<Unit>,
                #(#item_lv: #item_lt),*
            }

            impl<Unit, #(#item_lt),*> ::kompozit::Composition for #scope_gt<Unit, #(#item_lt),*>
            where
                Unit: ?Sized,
                #(#item_lt: ::kompozit::Composition<Unit = Unit>),*
            {
                type Unit = Unit;
                type Viewer<'s> = #viewer_gt<
                    Unit,
                    #(<#item_lt as ::kompozit::Composition>::Viewer<'s>),*
                > where
                    Self: 's,
                    Unit: 's,
                    #(#item_lt: 's),*;

                fn init() -> Self {
                    Self {
                        _unit: ::core::marker::PhantomData,
                        #(#item_lv: ::kompozit::Composition::init()),*
                    }
                }

                fn view<'s>(&'s mut self) -> <Self as ::kompozit::Composition>::Viewer<'s> {
                    return #viewer_gt {
                        _item: ::core::marker::PhantomData,
                        view_state: #viewer_state_gt::#begin_state_lt,
                        #(#item_lv: <#item_lt as ::kompozit::Composition>::view(&mut self.#item_lv)),*
                    }
                }
            }

            #[allow(non_camel_case_types)]
            struct #viewer_gt<Item: ?Sized, #(#item_lt),*> {
                _item: ::core::marker::PhantomData<Item>,
                view_state: #viewer_state_gt,
                #(#item_lv: #item_lt),*
            }

            #[allow(non_camel_case_types)]
            enum #viewer_state_gt {
                #(#state_lt),*
            }

            impl<Item, #(#item_lt),*> ::kompozit::Viewer for #viewer_gt<Item, #(#item_lt),*>
            where
                Item: ?Sized,
                #(
                    #item_lt: ::kompozit::Viewer<Item = Item>
                ),*
            {
                type Item = Item;

                fn move_next(&mut self) {
                    self.view_state = match self.view_state {
                        #(#viewer_state_gt::#state_lt => #viewer_state_gt::#next_state_lt,)*
                    };
                }

                fn move_prev(&mut self) {
                    self.view_state = match self.view_state {
                        #(#viewer_state_gt::#state_lt => #viewer_state_gt::#prev_state_lt),*
                    };
                }

                fn current(&mut self) -> ::core::option::Option<&mut <Self as ::kompozit::Viewer>::Item> {
                    match self.view_state {
                        | #viewer_state_gt::#begin_state_lt
                        | #viewer_state_gt::#end_state_lt => ::core::option::Option::None,
                        #(#viewer_state_gt::#item_lt => ::kompozit::Viewer::current(&mut self.#item_lv)),*
                    }
                }
            }

            #(#item)*

            #parent_prologue

            #[allow(non_snake_case)]
            let mut #scope_gv: &mut #scope_gt<_, #(#item_infer),*> = #parent_gv;
        }
    }

    pub fn epilogue(&self) -> proc_macro2::TokenStream {
        self.parent.epilogue()
    }
}

impl ComposeNode for ScopeItem {
    fn id(&self) -> EntityId<impl AsRef<str>> {
        EntityId {
            name: item_id_name(self.index),
            span: Span::mixed_site()
        }
    }

    fn prologue(&self) -> proc_macro2::TokenStream {
        let (item_gt, item_gv) = {
            let id = self.id();
            (id.gt(), id.gv())
        };

        let scope_gv = EntityId {
                name: scope_id_name(),
                span: Span::mixed_site(),
            }.gv();

        let composer_gv = EntityId {
            name: item_composer_id_name(self.index),
            span: Span::mixed_site(),
        }
        .gv();

        let default_slot_gv = EntityId {
            name: format!("scope item {} default slot", self.index),
            span: Span::mixed_site(),
        }
        .gv();

        if self.index == 0 {
            quote::quote_spanned! { Span::mixed_site() =>
                #[allow(non_snake_case)]
                let #composer_gv = ::kompozit::private::composer(&*#scope_gv);

                #[allow(non_snake_case)]
                let mut #item_gv = {
                    use ::core::convert::From;
                    #item_gt::from(&mut #scope_gv)
                };

                #[allow(non_snake_case)]
                let mut #default_slot_gv = ::kompozit::Composition::init();

                #[allow(non_snake_case)]
                let #item_gv = {
                    #[allow(unused_imports)]
                    use ::kompozit::private::ComposeTarget;
                    (&#composer_gv).compose(&mut #item_gv, &mut #default_slot_gv)
                };
            }
        } else {
            quote::quote_spanned! { Span::mixed_site() =>
                #[allow(non_snake_case)]
                let #composer_gv = ::kompozit::private::composer(&*#scope_gv);

                #[allow(non_snake_case)]
                let mut #item_gv = {
                    use ::core::convert::From;
                    #item_gt::from(&mut #scope_gv)
                };

                #[allow(non_snake_case)]
                let mut #default_slot_gv = ::kompozit::Composition::init();

                #[allow(non_snake_case)]
                let #item_gv = {
                    #[allow(unused_imports)]
                    use ::kompozit::private::{ComposeTarget, ComposeStub};
                    (&#composer_gv).compose(&mut #item_gv, &mut #default_slot_gv)
                };
            }
        }
    }

    fn epilogue(&self) -> proc_macro2::TokenStream {
        let composer_gv = EntityId {
            name: item_composer_id_name(self.index),
            span: Span::mixed_site(),
        }
        .gv();

            let item_gv = EntityId {
                name: item_id_name(self.index),
                span: Span::mixed_site().located_at(self.span),
            }.gv();

            let call = quote::quote_spanned! { Span::mixed_site().located_at(self.span) =>
                compose!(#composer_gv, #item_gv)
            };

            quote::quote_spanned! { Span::mixed_site() =>
                ::kompozit::#call;
            }
    }
}
