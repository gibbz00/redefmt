//! # redefmt-macros

// TEMP: should be part of next stable release (1.88)
// https://github.com/rust-lang/rust/pull/140514
#![feature(proc_macro_span)]
// TEMP:
#![allow(missing_docs)]

use proc_macro::TokenStream;

mod error;
pub(crate) use error::RedefmtMacroError;

mod db_client;
pub(crate) use db_client::{DbClients, db_clients, register_write_statement};

mod derive_format;

mod write_statement;

#[cfg(feature = "logger")]
mod print_statement;

mod statement_utils;
pub(crate) use statement_utils::StatementUtils;

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

#[cfg(feature = "logger")]
#[proc_macro]
pub fn print(token_stream: TokenStream) -> TokenStream {
    print_statement::print_macro_impl(token_stream, false)
}

#[cfg(feature = "logger")]
#[proc_macro]
pub fn println(token_stream: TokenStream) -> TokenStream {
    print_statement::print_macro_impl(token_stream, true)
}

#[cfg(feature = "logger")]
#[proc_macro]
pub fn log(token_stream: TokenStream) -> TokenStream {
    print_statement::log_macro_impl(token_stream)
}

#[cfg(feature = "logger")]
#[proc_macro]
pub fn trace(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, redefmt_core::codec::frame::Level::Trace)
}

#[cfg(feature = "logger")]
#[proc_macro]
pub fn debug(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, redefmt_core::codec::frame::Level::Debug)
}

#[cfg(feature = "logger")]
#[proc_macro]
pub fn info(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, redefmt_core::codec::frame::Level::Info)
}

#[cfg(feature = "logger")]
#[proc_macro]
pub fn warn(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, redefmt_core::codec::frame::Level::Warn)
}

#[cfg(feature = "logger")]
#[proc_macro]
pub fn error(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, redefmt_core::codec::frame::Level::Error)
}
