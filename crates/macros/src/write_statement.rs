use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use redefmt_args::provided_args::CombinedFormatString;
use redefmt_db::statement_table::write::WriteStatement;
use syn::{Token, parse_macro_input};

use crate::*;

pub fn macro_impl(token_stream: TokenStream, append_newline: bool) -> TokenStream {
    let WriteArgs { span, formatter_ident, combined_format_string } = parse_macro_input!(token_stream);

    let db_clients = db_clients!(span);

    let dynamic_arg_idents = combined_format_string
        .provided_args()
        .dynamic_args()
        .map(|dynamic_arg| match dynamic_arg.raw() {
            true => syn::Ident::new_raw(dynamic_arg.as_ref(), Span::call_site()),
            false => syn::Ident::new(dynamic_arg.as_ref(), Span::call_site()),
        })
        .collect::<Vec<_>>();

    let write_statement = WriteStatement::FormatString(combined_format_string);

    let write_id_expr = register_write_statement!(&db_clients, &write_statement, &formatter_ident, span);

    let append_newline_expr = append_newline.then(|| {
        quote! { '\n'.fmt(#formatter_ident) }
    });

    quote! {
        {
            use ::redefmt::Format as _;
            #write_id_expr;
            #(
              #dynamic_arg_idents.fmt(#formatter_ident);
            )*
            #append_newline_expr;
        }
    }
    .into()
}

pub struct WriteArgs {
    span: Span,
    formatter_ident: syn::Ident,
    combined_format_string: CombinedFormatString<'static>,
}

impl syn::parse::Parse for WriteArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let formatter_ident = input.parse()?;
        let _ = input.parse::<Token![,]>()?;
        let combined_format_string = input.parse()?;

        Ok(Self { span, formatter_ident, combined_format_string })
    }
}
