use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use redefmt_args::FormatExpression;
use redefmt_db::{
    Table,
    statement_table::{
        print::{Location, PrintInfo, PrintStatement},
        stored_format_expression::StoredFormatExpression,
    },
};
use syn::{parse::ParseStream, parse_macro_input};

use crate::*;

pub fn macro_impl(token_stream: TokenStream, append_newline: bool) -> TokenStream {
    let PrintArgs { span, print_info, format_expression } =
        parse_macro_input!(token_stream with PrintArgs::parse_print);

    let db_clients = db_clients!(span);

    let format_argument_expressions = format_expression
        .provided_args()
        .expressions()
        .cloned()
        .collect::<Vec<_>>();

    let print_statement = {
        let stored_expression = StoredFormatExpression::new(format_expression.into(), append_newline);
        PrintStatement::new(print_info, stored_expression)
    };

    let statement_id = match db_clients.crate_db.insert(&print_statement) {
        Ok(statement_id) => statement_id,
        Err(err) => {
            let macro_error = RedefmtMacroError::from(err);
            return macro_error.as_compiler_error(span);
        }
    };

    let crate_id_inner = db_clients.crate_id.as_ref();
    let statement_id_inner = statement_id.as_ref();

    quote! {
        {
            let mut global_logger_handle = ::redefmt::logger::GlobalLogger::write_start((
                ::redefmt::identifiers::CrateId::new(#crate_id_inner),
                ::redefmt::identifiers::PrintStatementId::new(#statement_id_inner)
            ));
            #(
                global_logger_handle.write_format(&#format_argument_expressions);
            )*
            global_logger_handle.write_end();
        }
    }
    .into()
}

pub struct PrintArgs {
    span: Span,
    print_info: PrintInfo<'static>,
    format_expression: FormatExpression<'static>,
}

impl PrintArgs {
    fn parse_print(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let location = {
            let rust_span = proc_macro::Span::call_site();
            let file = rust_span.file();
            let line = rust_span.start().line();
            Location::new(file, line as u32)
        };

        let print_info = PrintInfo::new(None, location);

        let format_expression = input.parse()?;

        Ok(Self { span, print_info, format_expression })
    }
}
