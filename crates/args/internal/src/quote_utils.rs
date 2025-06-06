use quote::{ToTokens, quote};

/// Default `ToTokens` implementation for `Option` only prints inner is option is some.
pub struct PrintOption<'a, T>(Option<&'a T>);

impl<'a, T> PrintOption<'a, T> {
    pub fn new(inner: &'a Option<T>) -> Self {
        Self(inner.as_ref())
    }
}

impl<T: ToTokens> ToTokens for PrintOption<'_, T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let option_tokens = match &self.0 {
            Some(inner) => quote! { Some(#inner) },
            None => quote! { None },
        };

        tokens.extend(option_tokens);
    }
}

#[cfg(test)]
pub fn assert_tokens<T: ToTokens>(input: T, expected: proc_macro2::TokenStream) {
    let actual = input.to_token_stream();

    // WORKAROUND: `TokenStream` doesn't implement `PartialEq`
    // https://github.com/rust-lang/rust/issues/51074
    let expected_debug = alloc::format!("{expected:?}");
    let actual_debug = alloc::format!("{actual:?}");

    assert_eq!(expected_debug, actual_debug);
}
