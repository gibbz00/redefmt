use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, TypeParamBound, parse_macro_input, spanned::Spanned};

use crate::*;

pub fn macro_impl(token_stream: TokenStream) -> TokenStream {
    let type_definition = parse_macro_input!(token_stream as DeriveInput);

    let db_clients = match DbClients::new() {
        Ok(clients) => clients,
        Err(err) => return err.as_syn_error(type_definition.span()).into_compile_error().into(),
    };

    let ident = type_definition.ident;

    let impl_body_result: syn::Result<TokenStream2> = match type_definition.data {
        Data::Struct(data_struct) => struct_impl(&db_clients, data_struct),
        Data::Enum(data_enum) => enum_impl(&db_clients, data_enum),
        Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "redefmt::Format derives on unions are not supported",
        )),
    };

    let impl_body = match impl_body_result {
        Ok(impl_body) => impl_body,
        Err(err) => return err.into_compile_error().into(),
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

fn struct_impl(db_clients: &DbClients, data_struct: DataStruct) -> syn::Result<TokenStream2> {
    match data_struct.fields {
        Fields::Unit => {
            // Register to DB
            todo!()
        }
        Fields::Named(fields_named) => todo!(),
        Fields::Unnamed(fields_unnamed) => todo!(),
    }
}

fn enum_impl(db_clients: &DbClients, enum_struct: DataEnum) -> syn::Result<TokenStream2> {
    todo!()
}
