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

// TEMP: should be part of next stable release (1.88)
// https://github.com/rust-lang/rust/pull/140514
#![feature(proc_macro_span)]

use std::fmt::Display;

use proc_macro::{Span, TokenStream, TokenTree};
use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned};

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
    match generate_tuples(token_stream, trait_name, impl_body_fn) {
        Ok(token_stream) => token_stream,
        Err(error) => error.as_compiler_error(),
    }
    .into()
}

fn generate_tuples(
    token_stream: TokenStream,
    trait_name: &str,
    impl_body_fn: ImplBodyFn,
) -> Result<TokenStream2, MacroError> {
    let max_tuple_length = parse_max_tuple_length(token_stream)?;

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
    trait_name: &proc_macro2::Ident,
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
    let tuple_indexes = (0..tuple_length).map(Literal::u8_unsuffixed);

    quote! {
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

        impl #generic_params_list private::Sealed for #tuple_type {}

        impl #generic_params_list Format for #tuple_type {
            fn fmt(&self, f: &mut Formatter) -> ::core::fmt::Result {
                f.write_deferred(self);
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
    let tuple_indexes = (0..tuple_length).map(Literal::u8_unsuffixed);

    quote! {
        impl #generic_params_list AsDeferredValue for #tuple_type {
            fn as_deferred_value(&self) -> DeferredValue {
                DeferredValue::Tuple(alloc::vec![
                    #( self. #tuple_indexes .as_deferred_value(), )*
                ])
            }
        }

        impl #generic_params_list private::Sealed for #tuple_type {}
    }
}

fn parse_max_tuple_length(token_stream: TokenStream) -> Result<u8, MacroError> {
    let Some(TokenTree::Literal(literal)) = token_stream.into_iter().next() else {
        return Err(MacroError::new("expected a literal", None));
    };

    let int = literal
        .to_string()
        .parse::<u8>()
        .map_err(|_| MacroError::new("invalid u8 literal", Some(literal.span())))?;

    let max_tuple_length = int;

    if max_tuple_length < MIN_TUPLE_LENGTH {
        return Err(MacroError::new(
            format!("{max_tuple_length} may not be less than {MIN_TUPLE_LENGTH}"),
            Some(literal.span()),
        ));
    }

    if max_tuple_length > MAX_TUPLE_LENGTH {
        return Err(MacroError::new(
            format!("{max_tuple_length} may not be greater than {MAX_TUPLE_LENGTH}"),
            Some(literal.span()),
        ));
    }

    Ok(max_tuple_length)
}

struct MacroError {
    message: String,
    span: Option<Span>,
}

impl MacroError {
    fn new(message: impl Display, span: Option<proc_macro::Span>) -> Self {
        Self { message: message.to_string(), span }
    }

    // taken from syn::ErrorMessage
    fn as_compiler_error(&self) -> TokenStream2 {
        let message = &self.message;
        let span = self.span.unwrap_or(Span::call_site()).into();

        quote_spanned! {span=> ::core::compile_error!(#message) }
    }
}
