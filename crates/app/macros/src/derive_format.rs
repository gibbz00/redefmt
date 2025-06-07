use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use redefmt_db::{
    Table,
    statement_table::type_structure::{StructVariant, TypeStructure, TypeStructureVariant},
};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, TypeParamBound, parse_macro_input, spanned::Spanned,
};

use crate::*;

pub fn macro_impl(token_stream: TokenStream) -> TokenStream {
    let type_definition = parse_macro_input!(token_stream as DeriveInput);

    let db_clients = db_clients!(type_definition.span());

    let ident = type_definition.ident;

    let (variant, impl_body) = match type_definition.data {
        Data::Struct(data_struct) => struct_impl(data_struct),
        Data::Enum(data_enum) => enum_impl(&ident, data_enum),
        Data::Union(data_union) => {
            return syn::Error::new(
                data_union.union_token.span,
                "redefmt::Format derives on unions are not supported",
            )
            .into_compile_error()
            .into();
        }
    };

    let type_structure = TypeStructure { name: ident.to_string().into(), variant };

    let id_pair = match db_clients.crate_db.insert(&type_structure) {
        Ok(statement_id) => {
            let crate_id_inner = db_clients.crate_id.as_ref();
            let statement_id_inner = statement_id.as_ref();
            let formatter_ident = format_ident!("f");

            ::quote::quote! {
                #formatter_ident.write((
                    ::redefmt::identifiers::CrateId::new(#crate_id_inner),
                    ::redefmt::identifiers::TypeStructureId::new(#statement_id_inner)
                ))
            }
        }
        Err(err) => {
            let macro_error = RedefmtMacroError::from(err);
            return macro_error.as_compiler_error(ident.span());
        }
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
            fn fmt(&self, f: &mut ::redefmt::Formatter) -> ::core::fmt::Result {
                #id_pair;
                #impl_body
                ::core::fmt::Result::Ok(())
            }
        }
    }
    .into()
}

fn struct_impl(data_struct: DataStruct) -> (TypeStructureVariant, TokenStream2) {
    let variant = struct_variant(&data_struct.fields);

    let impl_body = match data_struct.fields {
        Fields::Unit => quote! {},
        Fields::Named(fields_named) => {
            let field_idents = fields_named
                .named
                .into_iter()
                .map(|field| field.ident.expect("named field has no identifier"));

            quote! { #(self.#field_idents.fmt(f)?;)* }
        }
        Fields::Unnamed(fields_unnamed) => {
            let tuple_indexes = (0..fields_unnamed.unnamed.len()).map(syn::Index::from);

            quote! { #(self.#tuple_indexes.fmt(f)?;)* }
        }
    };

    (TypeStructureVariant::Struct(variant), impl_body)
}

fn enum_impl(ident: &Ident, enum_struct: DataEnum) -> (TypeStructureVariant, TokenStream2) {
    // skip generating a match statemetn to avoid `error[E0004]: non-exhaustive patterns`
    if enum_struct.variants.is_empty() {
        return (TypeStructureVariant::Enum(Vec::new()), quote! {});
    }

    let enum_variants = enum_struct
        .variants
        .iter()
        .map(|variant| {
            let name = variant.ident.to_string();
            let struct_variant = struct_variant(&variant.fields);

            (name, struct_variant)
        })
        .collect();

    let match_arms = enum_struct
        .variants
        .into_iter()
        .enumerate()
        .map(|(variant_index, variant)| {
            let variant_ident = variant.ident;

            match variant.fields {
                Fields::Named(fields_named) => {
                    let field_idents = fields_named
                        .named
                        .into_iter()
                        .map(|field| field.ident.expect("named field has no identifier"))
                        .collect::<Vec<_>>();

                    quote! {
                        #ident::#variant_ident { #(#field_idents),* } => {
                            f.write_raw(#variant_index);
                            #(#field_idents.fmt(f)?;)*
                        }
                    }
                }
                Fields::Unnamed(fields_unnamed) => {
                    let tuple_idents = (0..fields_unnamed.unnamed.len())
                        .map(|index| format_ident!("i_{}", index))
                        .collect::<Vec<_>>();

                    quote! {
                        #ident::#variant_ident(#(#tuple_idents),*) => {
                            f.write_raw(#variant_index);
                            #(#tuple_idents.fmt(f)?;)*
                        }
                    }
                }
                Fields::Unit => {
                    quote! {
                        #ident::#variant_ident => {
                            f.write_raw(#variant_index);
                        }
                    }
                }
            }
        });

    let impl_body = quote! {
        match self {
            #(#match_arms),*
        }
    };

    (TypeStructureVariant::Enum(enum_variants), impl_body)
}

fn struct_variant(fields: &Fields) -> StructVariant {
    match fields {
        Fields::Named(fields_named) => {
            let field_names = fields_named
                .named
                .iter()
                .map(|field| field.ident.as_ref().expect("named field has no identifier").to_string())
                .collect();

            StructVariant::Named(field_names)
        }
        Fields::Unnamed(fields_unnamed) => StructVariant::Tuple(fields_unnamed.unnamed.len() as u8),
        Fields::Unit => StructVariant::Unit,
    }
}
