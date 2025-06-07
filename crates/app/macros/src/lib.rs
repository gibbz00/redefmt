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
pub(crate) use db_client::{DbClients, db_clients};

mod derive_format;

mod write_statement;

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

#[proc_macro]
pub fn print(token_stream: TokenStream) -> TokenStream {
    print_statement::print_macro_impl(token_stream, false, false)
}

#[proc_macro]
pub fn print_compat(token_stream: TokenStream) -> TokenStream {
    print_statement::print_macro_impl(token_stream, true, false)
}

#[proc_macro]
pub fn println(token_stream: TokenStream) -> TokenStream {
    print_statement::print_macro_impl(token_stream, false, true)
}

#[proc_macro]
pub fn println_compat(token_stream: TokenStream) -> TokenStream {
    print_statement::print_macro_impl(token_stream, true, true)
}

#[proc_macro]
pub fn log(token_stream: TokenStream) -> TokenStream {
    print_statement::log_macro_impl(token_stream, false)
}

#[proc_macro]
pub fn log_compat(token_stream: TokenStream) -> TokenStream {
    print_statement::log_macro_impl(token_stream, true)
}

#[proc_macro]
pub fn trace(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, false, redefmt_core::frame::Level::Trace)
}

#[proc_macro]
pub fn trace_compat(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, true, redefmt_core::frame::Level::Trace)
}

#[proc_macro]
pub fn debug(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, false, redefmt_core::frame::Level::Debug)
}

#[proc_macro]
pub fn debug_compat(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, true, redefmt_core::frame::Level::Debug)
}

#[proc_macro]
pub fn info(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, false, redefmt_core::frame::Level::Info)
}

#[proc_macro]
pub fn info_compat(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, true, redefmt_core::frame::Level::Info)
}

#[proc_macro]
pub fn warn(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, false, redefmt_core::frame::Level::Warn)
}

#[proc_macro]
pub fn warn_compat(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, true, redefmt_core::frame::Level::Warn)
}

#[proc_macro]
pub fn error(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, false, redefmt_core::frame::Level::Error)
}

#[proc_macro]
pub fn error_compat(token_stream: TokenStream) -> TokenStream {
    print_statement::shorthand_log_macro_impl(token_stream, true, redefmt_core::frame::Level::Error)
}
