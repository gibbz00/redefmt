use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use redefmt_args::FormatExpression;
use redefmt_core::frame::Level;
use redefmt_db::{
    Table,
    statement_table::print::{Location, PrintStatement},
};
use syn::{Token, parse::ParseStream, parse_macro_input};

use crate::*;

struct Args {
    span: Span,
    level_expression: Option<syn::Expr>,
    format_expression: FormatExpression<'static>,
    compat_args_expression: Option<TokenStream2>,
}

macro_rules! parse_args {
    ($token_stream:ident, $fork_for_compat:expr, $with_level:expr) => {{
        match ($with_level, $fork_for_compat) {
            (true, false) => parse_macro_input!($token_stream with Args::parse_with_level),
            (true, true) => parse_macro_input!($token_stream with Args::parse_with_level_compat),
            (false, false) => parse_macro_input!($token_stream with Args::parse_no_level),
            (false, true) => parse_macro_input!($token_stream with Args::parse_no_level_compat),
        }
    }};
}

impl Args {
    fn parse_with_level(input: ParseStream) -> syn::Result<Self> {
        Self::parse_impl(true, false, input)
    }

    fn parse_with_level_compat(input: ParseStream) -> syn::Result<Self> {
        Self::parse_impl(true, true, input)
    }

    fn parse_no_level(input: ParseStream) -> syn::Result<Self> {
        Self::parse_impl(false, false, input)
    }

    fn parse_no_level_compat(input: ParseStream) -> syn::Result<Self> {
        Self::parse_impl(false, true, input)
    }

    fn parse_impl(with_level: bool, fork_for_compat: bool, input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let mut level_expression = None;
        let mut compat_args_expression = None;

        if with_level {
            level_expression = input.parse().map(Some)?;
            let _ = input.parse::<Token![,]>()?;
        }

        if fork_for_compat {
            compat_args_expression = input.fork().parse().map(Some)?;
        }

        let format_expression = input.parse()?;

        Ok(Self { span, level_expression, format_expression, compat_args_expression })
    }
}

pub fn print_macro_impl(token_stream: TokenStream, add_non_deferred: bool, append_newline: bool) -> TokenStream {
    let args = parse_args!(token_stream, add_non_deferred, false);
    try_macro_impl(args, append_newline)
}

pub fn log_macro_impl(token_stream: TokenStream, add_non_deferred: bool) -> TokenStream {
    let args = parse_args!(token_stream, add_non_deferred, true);
    try_macro_impl(args, true)
}

pub fn shorthand_log_macro_impl(token_stream: TokenStream, add_non_deferred: bool, level: Level) -> TokenStream {
    let mut args = parse_args!(token_stream, add_non_deferred, false);
    args.level_expression = Some(level_expression(level));
    try_macro_impl(args, true)
}

fn try_macro_impl(args: Args, append_newline: bool) -> TokenStream {
    let Args { span, level_expression, format_expression, compat_args_expression } = args;
    match macro_impl(
        level_expression,
        format_expression,
        compat_args_expression,
        append_newline,
    ) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.as_compiler_error(span),
    }
}

/// `level_expression` = None implies print statement
fn macro_impl(
    level_expression: Option<syn::Expr>,
    format_expression: FormatExpression,
    compat_args_expression: Option<TokenStream2>,
    append_newline: bool,
) -> Result<TokenStream2, RedefmtMacroError> {
    let db_clients = DbClients::new()?;

    let (stored_expression, provided_args) = StatementUtils::dissolve_expression(format_expression, append_newline);

    let print_statement = PrintStatement { location: location(), stored_expression };

    let statement_id = db_clients.crate_db.insert(&print_statement)?;

    let crate_id_inner = db_clients.crate_id.as_ref();
    let statement_id_inner = statement_id.as_ref();

    let maybe_log_level = match &level_expression {
        Some(expression) => quote! { Some(#expression) },
        None => quote! { None },
    };

    let non_deferred_expr = match (&level_expression, compat_args_expression) {
        (Some(level_expression), Some(log_args)) => {
            let log_compat_expr = quote! {
                ::redefmt::redefmt_to_log!(::redefmt::LevelCompat(#level_expression).into(), #log_args)
            };

            Some(log_compat_expr)
        }
        (None, Some(print_args)) => {
            let print_compat_expr = match append_newline {
                true => quote! { ::redefmt::redefmt_to_println!(#print_args) },
                false => quote! { ::redefmt::redefmt_to_print!(#print_args) },
            };

            Some(print_compat_expr)
        }
        (_, None) => None,
    };

    let deferred_expr = quote! {
        let mut global_logger_handle = ::redefmt::logger::GlobalLogger::write_start(
            (
                ::redefmt::identifiers::CrateId::new(#crate_id_inner),
                ::redefmt::identifiers::PrintStatementId::new(#statement_id_inner)
            ),
            #maybe_log_level
        );
        #(
            global_logger_handle.write_format(&(&#provided_args));
        )*
        global_logger_handle.write_end();
    };

    Ok(quote! {
        { #non_deferred_expr }
        { #deferred_expr }
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
