//! # redefmt-macros

// TEMP:
#![allow(missing_docs)]

use proc_macro::TokenStream;

mod error;
pub(crate) use error::RedefmtMacroError;

mod db_client;
pub(crate) use db_client::DbClients;

mod derive_format;

#[proc_macro_derive(Format, attributes(redefmt))]
pub fn derive_format(token_stream: TokenStream) -> TokenStream {
    derive_format::macro_impl(token_stream)
}
