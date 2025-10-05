mod composable;

use proc_macro_error::proc_macro_error;
use quote::ToTokens;

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

    quote::quote_spanned! { composition.span() =>
        #composer.check(#composition)
    }
    .into()
}

#[proc_macro_error]
#[proc_macro]
pub fn comp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut expr = syn::parse_macro_input!(input as syn::Expr);
    composable::transform(false, expr).to_token_stream().into()
}

#[proc_macro_error]
#[proc_macro]
pub fn comp_move(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut expr = syn::parse_macro_input!(input as syn::Expr);
    composable::transform(true, expr).to_token_stream().into()
}
