use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use redefmt_db::{
    Table,
    statement_table::write::{StructVariant, TypeStructure, TypeStructureVariant, WriteStatement},
};
use redefmt_internal::identifiers::{CrateId, WriteStatementId};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, TypeParamBound, parse_macro_input, spanned::Spanned,
};

use crate::*;

pub fn macro_impl(token_stream: TokenStream) -> TokenStream {
    let type_definition = parse_macro_input!(token_stream as DeriveInput);

    let db_clients = match DbClients::new() {
        Ok(clients) => clients,
        Err(err) => return err.as_compiler_error(type_definition.span()),
    };

    let ident = type_definition.ident;

    let impl_body_result = match type_definition.data {
        Data::Struct(data_struct) => struct_impl(&db_clients, &ident, data_struct),
        Data::Enum(data_enum) => enum_impl(&db_clients, &ident, data_enum),
        Data::Union(data_union) => {
            return syn::Error::new(
                data_union.union_token.span,
                "redefmt::Format derives on unions are not supported",
            )
            .into_compile_error()
            .into();
        }
    };

    let impl_body = match impl_body_result {
        Ok(impl_body) => impl_body,
        Err(err) => return err.as_compiler_error(ident.span()),
    };

    let mut generics = type_definition.generics;
    let (impl_generics, type_generics, where_clause) = {
        let bounds: TypeParamBound = syn::parse_str("::redefmt::Format").expect("valid type parameter bound");

        generics
            .type_params_mut()
            .for_each(|param| param.bounds.push(bounds.clone()));

        generics.split_for_impl()
    };

    quote! {
        impl #impl_generics ::redefmt::Format for #ident #type_generics #where_clause {
            fn fmt(&self, f: &mut ::redefmt::Formatter) {
                #impl_body
            }
        }
    }
    .into()
}

fn struct_impl(
    db_clients: &DbClients,
    ident: &Ident,
    data_struct: DataStruct,
) -> Result<TokenStream2, RedefmtMacroError> {
    match data_struct.fields {
        Fields::Unit => {
            let write_statement = WriteStatement::TypeStructure(TypeStructure {
                name: ident.to_string(),
                variant: TypeStructureVariant::Struct(StructVariant::Unit),
            });

            let write_statement_id = db_clients.crate_db.insert(&write_statement)?;

            let write_id = gen_write_id(db_clients.crate_id, write_statement_id);

            Ok(quote! {
                f.write(#write_id);
            })
        }
        Fields::Named(fields_named) => {
            let field_idents = fields_named
                .named
                .into_iter()
                .map(|field| field.ident.expect("named field has no identifier"));

            let field_names = field_idents.clone().map(|x| x.to_string()).collect();

            let write_statement = WriteStatement::TypeStructure(TypeStructure {
                name: ident.to_string(),
                variant: TypeStructureVariant::Struct(StructVariant::Named(field_names)),
            });

            let write_statement_id = db_clients.crate_db.insert(&write_statement)?;

            let write_id = gen_write_id(db_clients.crate_id, write_statement_id);

            Ok(quote! {
                f.write(#write_id);
                #(self.#field_idents.fmt(f);)*
            })
        }
        Fields::Unnamed(fields_unnamed) => todo!(),
    }
}

fn enum_impl(db_clients: &DbClients, ident: &Ident, enum_struct: DataEnum) -> Result<TokenStream2, RedefmtMacroError> {
    todo!()
}

fn gen_write_id(crate_id: CrateId, write_statement_id: WriteStatementId) -> TokenStream2 {
    let crate_id_inner = *crate_id.as_ref();
    let statement_id_inner = *write_statement_id.as_ref();

    quote! {
        (
            ::redefmt::identifiers::CrateId::new(#crate_id_inner),
            ::redefmt::identifiers::WriteStatementId::new(#statement_id_inner)
        )
    }
}
