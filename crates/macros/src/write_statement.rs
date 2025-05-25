use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use redefmt_args::FormatExpression;
use redefmt_db::statement_table::{stored_format_expression::StoredFormatExpression, write::WriteStatement};
use syn::{Token, parse_macro_input};

use crate::*;

pub fn macro_impl(token_stream: TokenStream, append_newline: bool) -> TokenStream {
    let WriteArgs { span, formatter_ident, format_expression } = parse_macro_input!(token_stream);

    let db_clients = db_clients!(span);

    let format_argument_expressions = format_expression
        .provided_args()
        .expressions()
        .cloned()
        .collect::<Vec<_>>();

    let write_id_expr = {
        let stored_expression = StoredFormatExpression::new(format_expression.into(), append_newline);
        let write_statement = WriteStatement::FormatExpression(stored_expression);
        register_write_statement!(&db_clients, &write_statement, &formatter_ident, span)
    };

    quote! {
        {
            use ::redefmt::Format as _;
            #write_id_expr;
            #[allow(unused_must_use)]
            {
                #(
                  #format_argument_expressions.fmt(#formatter_ident);
                )*
            }
        }
    }
    .into()
}

pub struct WriteArgs {
    span: Span,
    formatter_ident: syn::Ident,
    format_expression: FormatExpression<'static>,
}

impl syn::parse::Parse for WriteArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let formatter_ident = input.parse()?;
        let _ = input.parse::<Token![,]>()?;
        let format_expression = input.parse()?;

        Ok(Self { span, formatter_ident, format_expression })
    }
}
