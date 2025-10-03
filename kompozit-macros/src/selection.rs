use proc_macro2::Span;

use crate::ComposeNode;
use crate::id::EntityId;

pub struct Selection<Parent> {
    parent: Parent,
    span: proc_macro2::Span,
    name_prefix: String,
    variants: Vec<proc_macro2::Span>,
}

pub struct Variant {
    index: usize,
    span: proc_macro2::Span,
    name_prefix: String,
    selection_span: proc_macro2::Span,
}

fn selection_id_name(prefix: &str) -> String {
        format!("{} selection", prefix)
}

fn variant_id_name(prefix: &str, index: usize) -> String {
        format!("{} selection variant {index}", prefix)
}

fn variant_composer_id_name(prefix: &str, variant_index: usize) -> String {
format!("{} selection variant {} composer", prefix, variant_index)
}

impl<Parent: ComposeNode> Selection<Parent> {
    pub fn new(parent: Parent, span: proc_macro2::Span) -> Self {
        let name_prefix = parent.id().name.as_ref().to_owned();
        Self {
            span,
            name_prefix,
            parent,
            variants: vec![],
        }
    }

    pub fn add_variant(&mut self, span: proc_macro2::Span) -> Variant {
        self.variants.push(span);
        Variant {
            index: self.variants.len() - 1,
            name_prefix: self.parent.id().name.as_ref().to_owned(),
            span,
            selection_span: self.span,
        }
    }

    pub fn prologue(&self) -> proc_macro2::TokenStream {
        let (selection_gt, selection_gv, selection_lv) = {
            let id = EntityId {
                name: selection_id_name(&self.name_prefix),
                span: Span::mixed_site(),
            };
            (id.gt(), id.gv(), id.lv())
        };

        let variant_lt: Vec<_> = self
            .variants
            .iter()
            .enumerate()
            .map(|(index, _span)| 
            EntityId {
                name: variant_id_name(&self.name_prefix, index),
                span: Span::mixed_site(),
            }.lt()
                )
            .collect();
        let uninit_variant_lt = quote::format_ident!("Uninit");
        let variant_infer = vec![quote::quote! { _ }; self.variants.len()];

        let viewer_gt = EntityId {
            name: format!("{} selection viewer", self.name_prefix),
            span: self.span,
        }
        .gt();

        let parent_gv = self.parent.id().gv();
        let parent_prologue = self.parent.prologue();

        let variant = self.variants.iter().enumerate().map(|(index, span)| {
            let id = EntityId {
                name: variant_id_name(&self.name_prefix, index),
                span: Span::mixed_site(),
            };
            let current_variant_lt = id.lt();
            let current_variant_gt = id.gt();

        quote::quote_spanned! { Span::mixed_site() =>
                #[allow(non_camel_case_types)]
                struct #current_variant_gt<'s, 'ss, Unit: ?Sized, #(#variant_lt: ::kompozit::Composition),*> {
                    #selection_lv: &'s mut &'ss mut #selection_gt<Unit, #(#variant_lt),*>
                }

                impl<'s, 'ss, Unit, #(#variant_lt),*> ::core::convert::From<&'s mut &'ss mut #selection_gt<Unit, #(#variant_lt),*>> 
                    for #current_variant_gt<'s, 'ss, Unit, #(#variant_lt),*> 
                    where
                        Unit: ?Sized,
                        #(#variant_lt: ::kompozit::Composition),*
                    {
                    fn from(#selection_lv: &'s mut &'ss mut #selection_gt<Unit, #(#variant_lt),*>) -> Self {
                        Self {
                            #selection_lv,
                        }
                    }
                }

                impl<'s, 'ss, Unit, #(#variant_lt),*> ::kompozit::private::Slot
                    for #current_variant_gt<'s, 'ss, Unit, #(#variant_lt),*>
                    where
                        Unit: ?Sized,
                        #(#variant_lt: ::kompozit::Composition),*
                {
                    type Source = &'s mut &'ss mut #selection_gt<Unit, #(#variant_lt),*>;
                    type Target = #current_variant_lt;

                    fn get(&mut self) -> &mut Self::Target {
                        let #selection_lv: &mut #selection_gt<Unit, #(#variant_lt),*> = *self.#selection_lv;
                        match #selection_lv {
                            #selection_gt::#current_variant_lt(variant) => variant,
                            variant => {
                                *variant = #selection_gt::#current_variant_lt(<#current_variant_lt as ::kompozit::Composition>::init());
                                match variant {
                                    #selection_gt::#current_variant_lt(variant) => variant,
                                    _ => unreachable!(),
                                }
                            }
                        }
                    }
                }
            }
        });

        quote::quote_spanned! { Span::mixed_site() =>
            #[allow(non_camel_case_types)]
            enum #selection_gt<Unit: ?Sized, #(#variant_lt: ::kompozit::Composition),*> {
                #uninit_variant_lt(::core::marker::PhantomData<Unit>),
                #(#variant_lt(#variant_lt)),*
            }

            impl<Unit, #(#variant_lt),*> ::kompozit::Composition for #selection_gt<Unit, #(#variant_lt),*>
            where
                Unit: ?Sized,
                #(
                    #variant_lt: ::kompozit::Composition<Unit = Unit>
                ),*
            {
                type Unit = Unit;
                type Viewer<'s> = #viewer_gt<
                    Unit,
                    #(<#variant_lt as ::kompozit::Composition>::Viewer<'s>),*
                > where
                    Self: 's,
                    Unit: 's,
                    #(#variant_lt: 's),*;

                fn init() -> Self {
                    Self::#uninit_variant_lt(::core::marker::PhantomData)
                }

                fn view<'s>(&'s mut self) -> <Self as ::kompozit::Composition>::Viewer<'s> {
                    match self {
                        Self::#uninit_variant_lt(_) => #viewer_gt::#uninit_variant_lt(::core::marker::PhantomData),
                        #(Self::#variant_lt(variant) => #viewer_gt::#variant_lt(<#variant_lt as ::kompozit::Composition>::view(variant))),*
                    }
                }
            }

            #[allow(non_camel_case_types)]
            enum #viewer_gt<Item: ?Sized, #(#variant_lt),*> {
                #uninit_variant_lt(::core::marker::PhantomData<Item>),
                #(#variant_lt(#variant_lt)),*
            }

            impl<Item, #(#variant_lt),*> ::kompozit::Viewer for #viewer_gt<Item, #(#variant_lt),*>
            where
                Item: ?Sized,
                #(
                    #variant_lt: ::kompozit::Viewer<Item = Item>
                ),*
            {
                type Item = Item;

                fn move_next(&mut self) {
                    match self {
                        Self::#uninit_variant_lt(_) => {},
                        #(Self::#variant_lt(variant) => <#variant_lt as ::kompozit::Viewer>::move_next(variant)),*
                    }
                }

                fn move_prev(&mut self) {
                    match self {
                        Self::#uninit_variant_lt(_) => {},
                        #(Self::#variant_lt(variant) => <#variant_lt as ::kompozit::Viewer>::move_prev(variant)),*
                    }
                }

                fn current(&mut self) -> ::core::option::Option<&mut <Self as ::kompozit::Viewer>::Item> {
                    match self {
                        Self::#uninit_variant_lt(_) => ::core::option::Option::None,
                        #(Self::#variant_lt(variant) => <#variant_lt as ::kompozit::Viewer>::current(variant)),*
                    }
                }
            }

            #(#variant)*

            #parent_prologue

            #[allow(non_snake_case)]
            let mut #selection_gv: &mut #selection_gt<_, #(#variant_infer),*> = #parent_gv;
        }
    }

    pub fn epilogue(&self) -> proc_macro2::TokenStream {
        self.parent.epilogue()
    }
}

impl ComposeNode for Variant {
    fn id(&self) -> EntityId<impl AsRef<str>> {
        EntityId {
            name: variant_id_name(&self.name_prefix, self.index),
            span: Span::mixed_site(),
        }
    }

    fn prologue(&self) -> proc_macro2::TokenStream {
        let (variant_gt, variant_gv) = {
            let id = self.id();
            (id.gt(), id.gv())
        };

        let selection_gv = EntityId {
            name: selection_id_name(&self.name_prefix),
            span: Span::mixed_site(),
        }.gv();

        let composer_gv = EntityId {
            name: variant_composer_id_name(&self.name_prefix, self.index),
            span: Span::mixed_site(),
        }
        .gv();

        let default_slot_gv = EntityId {
            name: format!("{} selection variant {} default slot", self.name_prefix, self.index),
            span: Span::mixed_site(),
        }
        .gv();

        if self.index == 0 {
            quote::quote_spanned! { Span::mixed_site() =>
                #[allow(non_snake_case)]
                let mut #composer_gv = ::kompozit::private::composer(&*#selection_gv);

                #[allow(non_snake_case)]
                let mut #variant_gv = {
                    use ::core::convert::From;
                    #variant_gt::from(&mut #selection_gv)
                };

                #[allow(non_snake_case)]
                let mut #default_slot_gv = ::kompozit::Composition::init();

                #[allow(non_snake_case)]
                let #variant_gv = {
                    #[allow(unused_imports)]
                    use ::kompozit::private::ComposeTarget;
                    (&#composer_gv).compose(&mut #variant_gv, &mut #default_slot_gv)
                };
            }
        } else {
            quote::quote_spanned! { Span::mixed_site() =>
                #[allow(non_snake_case)]
                let mut #composer_gv = ::kompozit::private::composer(&*#selection_gv);

                #[allow(non_snake_case)]
                let mut #variant_gv = {
                    use ::core::convert::From;
                    #variant_gt::from(&mut #selection_gv)
                };

                #[allow(non_snake_case)]
                let mut #default_slot_gv = ::kompozit::Composition::init();

                #[allow(non_snake_case)]
                let #variant_gv = {
                    #[allow(unused_imports)]
                    use ::kompozit::private::{ComposeTarget, ComposeStub};
                    (&#composer_gv).compose(&mut #variant_gv, &mut #default_slot_gv)
                };
            }
        }
    }
    
    fn epilogue(&self) -> proc_macro2::TokenStream {
        let composer_gv = EntityId {
            name: variant_composer_id_name(&self.name_prefix, self.index),
            span: Span::mixed_site(),
        }
        .gv();

            let variant_gv = EntityId {
                name: variant_id_name(&self.name_prefix, self.index),
                span: Span::mixed_site().located_at(self.span),
                // span: Span::mixed_site(),
            }.gv();

            let call = quote::quote_spanned! { Span::mixed_site().located_at(self.span) =>
                compose!(#composer_gv, #variant_gv)
            };

            quote::quote_spanned! { Span::mixed_site() =>
                ::kompozit::#call;
            }
    }
}
