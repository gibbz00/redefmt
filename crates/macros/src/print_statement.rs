use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use redefmt_args::FormatExpression;
use redefmt_core::codec::frame::Level;
use redefmt_db::{
    Table,
    statement_table::print::{Location, PrintStatement},
};
use syn::{Token, parse::ParseStream, parse_macro_input};

use crate::*;

struct Args {
    span: Span,
    level_expression: Option<syn::Expr>,
    format_expression: FormatExpression<'static, syn::Expr>,
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
    format_expression: FormatExpression<syn::Expr>,
    append_newline: bool,
) -> Result<TokenStream2, RedefmtMacroError> {
    let db_clients = DbClients::new()?;

    let (deferred_format_string, provided_args) = format_expression.dissolve();

    let (provided_args, provided_identifiers) = provided_args.dissolve_expressions();

    let stored_expression = StatementUtils::prepare_stored(
        deferred_format_string,
        provided_args.len(),
        provided_identifiers,
        append_newline,
    );

    let print_statement = PrintStatement { location: location(), stored_expression };

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
                global_logger_handle.write_format(&#provided_args);
            )*
            global_logger_handle.write_end();
        }
    })
}

fn location() -> Location<'static> {
    let rust_span = proc_macro::Span::call_site();
    let file = rust_span.file().into();
    let line = rust_span.start().line() as u32;
    Location { file, line }
}

fn level_expression(level: Level) -> syn::Expr {
    let tokens = match level {
        Level::Trace => quote! { ::redefmt::Level::Trace },
        Level::Debug => quote! { ::redefmt::Level::Debug },
        Level::Info => quote! { ::redefmt::Level::Info },
        Level::Warn => quote! { ::redefmt::Level::Warn },
        Level::Error => quote! { ::redefmt::Level::Error },
    };

    syn::Expr::Verbatim(tokens)
}
