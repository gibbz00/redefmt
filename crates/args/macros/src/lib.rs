//! # `redefmt-args-macros`

#![no_std]
// TEMP:
#![allow(missing_docs)]

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use redefmt_args_internal::{FormatExpression, format_string::FormatString};
use syn::parse_macro_input;

#[proc_macro]
pub fn format_string(token_stream: TokenStream) -> TokenStream {
    parse_macro_input!(token_stream as FormatString)
        .to_token_stream()
        .into()
}

#[proc_macro]
pub fn processed_format_string(token_stream: TokenStream) -> TokenStream {
    let expression = parse_macro_input!(token_stream as FormatExpression);
    expression.processed_format_string.to_token_stream().into()
}

#[proc_macro]
pub fn deferred_format(token_stream: TokenStream) -> TokenStream {
    let FormatExpression { processed_format_string, provided_args } = parse_macro_input!(token_stream);

    let positional_args = provided_args.positional.into_iter().map(|expr| {
        quote! {{
            use ::redefmt_args::deferred::AsDeferredValue as _;
            (#expr).as_deferred_value()
        }}
    });

    let named_args = provided_args.named.into_iter().map(|(identifier, expr)| {
        quote! {
            (
                #identifier,
                { use ::redefmt_args::deferred::AsDeferredValue as _; (#expr).as_deferred_value() }
            )
        }
    });

    quote! {
        (
            #processed_format_string,
            ::redefmt_args::deferred::DeferredValues::new(
                [#(#positional_args),*],
                [#(#named_args),*]
            )
        )
    }
    .into()
}
