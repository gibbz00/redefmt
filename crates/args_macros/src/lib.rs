//! # `redefmt-args-macros`

#![no_std]
// TEMP:
#![allow(missing_docs)]

use proc_macro::TokenStream;
use quote::ToTokens;
use redefmt_args_internal::{FormatString, MappedFormatExpression};
use syn::parse_macro_input;

#[proc_macro]
pub fn format_string(token_stream: TokenStream) -> TokenStream {
    parse_macro_input!(token_stream as FormatString)
        .to_token_stream()
        .into()
}

#[proc_macro]
pub fn mapped_format_expression(token_stream: TokenStream) -> TokenStream {
    parse_macro_input!(token_stream as MappedFormatExpression)
        .to_token_stream()
        .into()
}
