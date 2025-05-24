//! # `redefmt-args-macros`

#![no_std]
// TEMP:
#![allow(missing_docs)]

use proc_macro::TokenStream;
use quote::ToTokens;
use redefmt_args_internal::{
    FormatString,
    provided_args::{CombinedFormatString, ProvidedArgs},
};
use syn::parse_macro_input;

#[proc_macro]
pub fn format_string(token_stream: TokenStream) -> TokenStream {
    parse_macro_input!(token_stream as FormatString)
        .to_token_stream()
        .into()
}

#[proc_macro]
pub fn provided_args(token_stream: TokenStream) -> TokenStream {
    parse_macro_input!(token_stream as ProvidedArgs)
        .to_token_stream()
        .into()
}

#[proc_macro]
pub fn combined_format_string(token_stream: TokenStream) -> TokenStream {
    parse_macro_input!(token_stream as CombinedFormatString)
        .to_token_stream()
        .into()
}
