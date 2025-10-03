mod id;
mod patcher;
mod scope;
mod selection;

use proc_macro_error::proc_macro_error;
use syn::spanned::Spanned;

const NAMESPACE: &str = "N_";

trait ComposeNode {
    fn id(&self) -> id::EntityId<impl AsRef<str>>;
    //
    // fn span(&self) -> proc_macro2::Span;

    fn prologue(&self) -> proc_macro2::TokenStream;

    fn epilogue(&self) -> proc_macro2::TokenStream;
}

struct RootScope {
    span: proc_macro2::Span,
}

impl ComposeNode for RootScope {
    fn id(&self) -> id::EntityId<impl AsRef<str>> {
        id::EntityId {
            name: "root scope",
            // span: proc_macro2::Span::mixed_site(),
            span: self.span,
        }
    }

    // fn span(&self) -> proc_macro2::Span {
    //     self.span
    // }

    fn prologue(&self) -> proc_macro2::TokenStream {
        quote::quote! {}
    }

    fn epilogue(&self) -> proc_macro2::TokenStream {
        quote::quote! {}
    }
}

/// fooobar
#[proc_macro]
pub fn compose(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct Input {
        composer: syn::Ident,
        comma_token: syn::Token![,],
        composition: syn::Ident,
    }

    impl syn::parse::Parse for Input {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(Self {
                composer: input.parse()?,
                comma_token: input.parse()?,
                composition: input.parse()?,
            })
        }
    }
    let input = syn::parse_macro_input!(input as Input);
    let composer = input.composer;
    let composition = input.composition;
    // let path = quote::quote_spanned! { comp.span().located_at(proc_macro2::Span::mixed_site()) =>
    //         };

    quote::quote_spanned! { composition.span() =>
        #composer.check(#composition)
    }
    .into()
}

#[proc_macro_error]
#[proc_macro]
pub fn comp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut block = syn::parse_macro_input!(input as syn::Block);

    // let root_scope = RootScope { span: block.span() };
    let root_scope = RootScope {
        // span: proc_macro2::Span::mixed_site().located_at(block.span()),
        span: proc_macro2::Span::call_site(),
    };
    let root_scope_gv = root_scope.id().gv();

    // let mut patcher = patcher::Patcher {
    //     scope: scope::Scope::new(root_scope, block.span()),
    // };
    let mut patcher = patcher::Patcher {
        scope: scope::Scope::new(
            root_scope,
            block.span(),
            // proc_macro2::Span::mixed_site().located_at(block.span()),
            // proc_macro2::Span::call_site(),
        ),
    };

    syn::visit_mut::VisitMut::visit_block_mut(&mut patcher, &mut block);
    let prologue = patcher.scope.prologue();

    // let recomp = quote::format_ident!("recomp", span = block.span());
    let recomp = quote::format_ident!("recomp", span = proc_macro2::Span::call_site());

    // let cast = quote::quote_spanned! { block.span() =>
    let cast = quote::quote_spanned! { proc_macro2::Span::call_site() =>
        #[allow(unused_imports)]
        use ::kompozit::private::{Caster, CastNever, Cast};
        let caster = &&Caster::new(&#recomp);
        caster.cast(#recomp)
    };

    // quote::quote_spanned! { block.span() =>
    quote::quote_spanned! { proc_macro2::Span::call_site() =>
        {
            let #recomp = ::kompozit::from_fn(|#[allow(non_snake_case)] #root_scope_gv: &mut _| {
                #prologue
                #block
            });

            #cast
        }

    }
    .into()
}
//
// #[proc_macro_error]
// #[proc_macro_attribute]
// pub fn composable(
//     attrs: proc_macro::TokenStream,
//     input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     input
// }
