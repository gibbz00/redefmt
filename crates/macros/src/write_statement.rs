use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use redefmt_args::FormatExpression;
use redefmt_db::{Table, statement_table::write::WriteStatement};
use syn::{Token, parse_macro_input};

use crate::*;

pub fn macro_impl(token_stream: TokenStream, append_newline: bool) -> TokenStream {
    let WriteArgs { span, statement_writer_ident, format_expression } = parse_macro_input!(token_stream);

    let db_clients = db_clients!(span);

    let (stored_expression, provided_args) = StatementUtils::dissolve_expression(format_expression, append_newline);

    let write_statement = WriteStatement(stored_expression);

    let statement_id = match db_clients.crate_db.insert(&write_statement) {
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
            use ::redefmt::Format as _;

            #statement_writer_ident.write_statement_id(
                ::redefmt::identifiers::CrateId::new(#crate_id_inner),
                ::redefmt::identifiers::WriteStatementId::new(#statement_id_inner)
            );

            #[allow(unused_must_use)]
            {
                #(
                  #provided_args.fmt(#statement_writer_ident.formatter());
                )*
            }
        }
    }
    .into()
}

pub struct WriteArgs {
    span: Span,
    statement_writer_ident: syn::Ident,
    format_expression: FormatExpression<'static>,
}

impl syn::parse::Parse for WriteArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let statement_writer_ident = input.parse()?;
        let _ = input.parse::<Token![,]>()?;
        let format_expression = input.parse()?;

        Ok(Self { span, statement_writer_ident, format_expression })
    }
}
