//! # redefmt-internal-macros - As the name suggests
//!
//! Generate tuple implementations for all permitations `&dyn T` and `impl ! T`.
//!
//! Macros take a max tuple length integer literal parameter that should be between 2
//! and 10. Upper limit exists to prevent excessive code generation.
//!
//! A max tuple length of N results in the geometric sum of 2^N tuple trait
//! implementations (= 2^(N+1) - 2) in order to account for all combinations
//! of `&dyn T` and `impl T`. N = 7 results in other words in the generation of
//! 254 trait implementations.
//!
//! Increasing the max tuple length from 7 to 10 may increase crate build time
//! by almost three fold. From ~2.4s to ~7.1s on the authors computer. Codegen
//! duration remains the same at about 0.1s. Not pulling in this proc macro
//! at all removes about 0.2s from cold compile times. Simply moving this to a
//! build script is therefore a slightly premature optimization given that it is
//! generated code itself which takes the longest to compile.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Ident, LitInt, parse_macro_input};

const MIN_TUPLE_LENGTH: u8 = 2;
const MAX_TUPLE_LENGTH: u8 = 10;

type ImplBodyFn = fn(u8, TokenStream2, TokenStream2) -> TokenStream2;

/// Se crate level documentation.
#[proc_macro]
pub fn impl_tuple_write_value(token_stream: TokenStream) -> TokenStream {
    try_impl_tuple(token_stream, "WriteValue", impl_write_value_body)
}

/// Se crate level documentation.
#[proc_macro]
pub fn impl_tuple_as_deferred_value(token_stream: TokenStream) -> TokenStream {
    try_impl_tuple(token_stream, "AsDeferredValue", impl_as_deferred_value_body)
}

fn try_impl_tuple(token_stream: TokenStream, trait_name: &str, impl_body_fn: ImplBodyFn) -> TokenStream {
    let lit_int = parse_macro_input!(token_stream as LitInt);

    match generate_tuples(lit_int, trait_name, impl_body_fn) {
        Ok(token_stream) => token_stream,
        Err(error) => error.into_compile_error(),
    }
    .into()
}

fn generate_tuples(lit_int: LitInt, trait_name: &str, impl_body_fn: ImplBodyFn) -> syn::Result<TokenStream2> {
    let max_tuple_length = lit_int.base10_parse::<u8>()?;

    if max_tuple_length < MIN_TUPLE_LENGTH {
        return Err(syn::Error::new(
            lit_int.span(),
            format!("{max_tuple_length} may not be less than {MIN_TUPLE_LENGTH}"),
        ));
    }

    if max_tuple_length > MAX_TUPLE_LENGTH {
        return Err(syn::Error::new(
            lit_int.span(),
            format!("{max_tuple_length} may not be greater than {MAX_TUPLE_LENGTH}"),
        ));
    }

    let trait_name = format_ident!("{trait_name}");

    // Geometric sum of 2^N
    let capacity = 2u16.pow((max_tuple_length + 1) as u32) - 2;
    let mut trait_impls = Vec::with_capacity(capacity as usize);

    for tuple_length in MIN_TUPLE_LENGTH..(max_tuple_length + 1) {
        generate_tuple_permutations(&mut trait_impls, tuple_length, &trait_name, impl_body_fn);
    }

    Ok(quote! {
        #(
            #trait_impls
        )*
    })
}

fn generate_tuple_permutations(
    trait_impl_buffer: &mut Vec<TokenStream2>,
    tuple_length: u8,
    trait_name: &Ident,
    impl_body_fn: ImplBodyFn,
) {
    for permutation in 0..2u16.pow(tuple_length as u32) {
        let generic_params = (0..permutation.count_ones())
            .map(|index| format_ident!("T{}", index))
            .collect::<Vec<_>>();

        let mut current_generic_index = 0;
        let tuple_type_indexes = (0..tuple_length).map(|index| match is_dyn(permutation, index) {
            true => quote! { &dyn #trait_name },
            false => {
                let ident = &generic_params[current_generic_index];
                current_generic_index += 1;
                quote! { #ident }
            }
        });

        let tuple_type = quote! { (#(#tuple_type_indexes),*) };

        let generic_params_list = quote! { <#(#generic_params: #trait_name),*> };

        let trait_impl = impl_body_fn(tuple_length, generic_params_list, tuple_type);

        trait_impl_buffer.push(trait_impl);
    }
}

fn is_dyn(permutation: u16, tuple_index: u8) -> bool {
    (permutation >> tuple_index & 1) == 0
}

fn impl_write_value_body(
    tuple_length: u8,
    generic_params_list: TokenStream2,
    tuple_type: TokenStream2,
) -> TokenStream2 {
    let tuple_indexes = (0..tuple_length as usize).map(syn::Index::from);

    quote! {
        #[::sealed::sealed]
        impl #generic_params_list WriteValue for #tuple_type {
            fn hint(&self) -> TypeHint {
                TypeHint::Tuple
            }

            fn write_raw(&self, dispatcher: &mut dyn Dispatcher) {
                #tuple_length.write_raw(dispatcher);

                #(
                    self. #tuple_indexes .write_value(dispatcher);
                )*
            }
        }

        impl #generic_params_list Format for #tuple_type {
            fn fmt(&self, f: &mut Formatter) -> ::core::fmt::Result {
                f.write(self);
                Ok(())
            }
        }
    }
}

fn impl_as_deferred_value_body(
    tuple_length: u8,
    generic_params_list: TokenStream2,
    tuple_type: TokenStream2,
) -> TokenStream2 {
    let tuple_indexes = (0..tuple_length as usize).map(syn::Index::from);

    quote! {
        #[::sealed::sealed]
        impl #generic_params_list AsDeferredValue for #tuple_type {
            fn as_deferred_value(&self) -> DeferredValue {
                DeferredValue::Tuple(alloc::vec![
                    #( self. #tuple_indexes .as_deferred_value(), )*
                ])
            }
        }
    }
}
