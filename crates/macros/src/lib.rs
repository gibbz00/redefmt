//! # redefmt-macros

// TEMP:
#![allow(missing_docs)]

use proc_macro::TokenStream;

mod error;
pub(crate) use error::RedefmtMacroError;

mod db_client;
pub(crate) use db_client::DbClients;

mod derive_format;

mod write_statement;

mod print_statement;

#[proc_macro_derive(Format, attributes(redefmt))]
pub fn derive_format(token_stream: TokenStream) -> TokenStream {
    derive_format::macro_impl(token_stream)
}

#[proc_macro]
pub fn write(token_stream: TokenStream) -> TokenStream {
    write_statement::macro_impl(token_stream, false)
}

#[proc_macro]
pub fn writeln(token_stream: TokenStream) -> TokenStream {
    write_statement::macro_impl(token_stream, true)
}

#[proc_macro]
pub fn print(token_stream: TokenStream) -> TokenStream {
    print_statement::macro_impl(token_stream, false)
}

#[proc_macro]
pub fn println(token_stream: TokenStream) -> TokenStream {
    print_statement::macro_impl(token_stream, true)
}
