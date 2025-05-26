use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use redefmt_args::FormatExpression;
use redefmt_core::codec::frame::Level;
use redefmt_db::{
    Table,
    statement_table::{
        print::{Location, PrintStatement},
        stored_format_expression::StoredFormatExpression,
    },
};
use syn::{Token, parse::ParseStream, parse_macro_input};

use crate::*;

struct Args {
    span: Span,
    level_expression: Option<syn::Expr>,
    format_expression: FormatExpression<'static>,
}

impl Args {
    fn parse_with_level(input: ParseStream) -> syn::Result<Self> {
        Self::parse_impl(true, input)
    }

    fn parse_no_level(input: ParseStream) -> syn::Result<Self> {
        Self::parse_impl(false, input)
    }

    fn parse_impl(with_level: bool, input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let mut level_expression = None;

        if with_level {
            level_expression = Some(input.parse()?);
            let _ = input.parse::<Token![,]>()?;
        }

        let format_expression = input.parse()?;

        Ok(Self { span, level_expression, format_expression })
    }
}

pub fn print_macro_impl(token_stream: TokenStream, append_newline: bool) -> TokenStream {
    let args = parse_macro_input!(token_stream with Args::parse_no_level);
    try_macro_impl(args, append_newline)
}

pub fn log_macro_impl(token_stream: TokenStream) -> TokenStream {
    let args = parse_macro_input!(token_stream with Args::parse_with_level);
    try_macro_impl(args, true)
}

pub fn shorthand_log_macro_impl(token_stream: TokenStream, level: Level) -> TokenStream {
    let mut args = parse_macro_input!(token_stream with Args::parse_no_level);
    args.level_expression = Some(level_expression(level));
    try_macro_impl(args, true)
}

fn try_macro_impl(args: Args, append_newline: bool) -> TokenStream {
    let Args { span, level_expression, format_expression } = args;
    match macro_impl(level_expression, format_expression, append_newline) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.as_compiler_error(span),
    }
}

fn macro_impl(
    level_expression: Option<syn::Expr>,
    format_expression: FormatExpression,
    append_newline: bool,
) -> Result<TokenStream2, RedefmtMacroError> {
    let db_clients = DbClients::new()?;

    let format_argument_expressions = format_expression
        .provided_args()
        .expressions()
        .cloned()
        .collect::<Vec<_>>();

    let print_statement = {
        let location = location();
        let stored_expression = StoredFormatExpression::new(format_expression.into(), append_newline);
        PrintStatement::new(location, stored_expression)
    };

    let statement_id = db_clients.crate_db.insert(&print_statement)?;

    let crate_id_inner = db_clients.crate_id.as_ref();
    let statement_id_inner = statement_id.as_ref();

    let maybe_log_level = match level_expression {
        Some(expression) => quote! { Some(#expression) },
        None => quote! { None },
    };

    Ok(quote! {
        {
            let mut global_logger_handle = ::redefmt::logger::GlobalLogger::write_start(
                (
                    ::redefmt::identifiers::CrateId::new(#crate_id_inner),
                    ::redefmt::identifiers::PrintStatementId::new(#statement_id_inner)
                ),
                #maybe_log_level
            );
            #(
                global_logger_handle.write_format(&#format_argument_expressions);
            )*
            global_logger_handle.write_end();
        }
    })
}

fn location() -> Location<'static> {
    let rust_span = proc_macro::Span::call_site();
    let file = rust_span.file();
    let line = rust_span.start().line();
    Location::new(file, line as u32)
}

fn level_expression(level: Level) -> syn::Expr {
    let tokens = match level {
        Level::Trace => quote! { ::redefmt::Level::Trace },
        Level::Debug => quote! { ::redefmt::Level::Trace },
        Level::Info => quote! { ::redefmt::Level::Trace },
        Level::Warn => quote! { ::redefmt::Level::Trace },
        Level::Error => quote! { ::redefmt::Level::Trace },
    };

    syn::Expr::Verbatim(tokens)
}
