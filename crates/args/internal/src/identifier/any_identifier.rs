use alloc::borrow::Cow;
use core::fmt::{Debug, Display};

use super::utils::RAW_START;
use crate::*;

/// Identifier or keyword which may be raw ('r#') prefixed
///
/// Not allowed in format strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnyIdentifier<'a> {
    pub(super) raw: bool,
    pub(super) inner: Cow<'a, str>,
}

impl<'a> AnyIdentifier<'a> {
    pub fn raw(&self) -> bool {
        self.raw
    }

    #[doc(hidden)]
    pub const unsafe fn new_unchecked(raw: bool, inner: &'a str) -> Self {
        Self { raw, inner: Cow::Borrowed(inner) }
    }

    pub(crate) fn unraw(self) -> ArgumentIdentifier<'a> {
        ArgumentIdentifier { inner: self.inner }
    }

    pub fn parse(cow_str: impl Into<Cow<'a, str>>) -> Result<Self, FormatStringParseError> {
        let cow_str = cow_str.into();

        if cow_str.is_empty() {
            return Err(FormatStringParseError::new(0, 0..0, IdentifierParseError::Empty));
        }

        let (offset, ident, raw) = match cow_str.strip_prefix(RAW_START) {
            Some(raw_ident) => (RAW_START.len(), raw_ident, true),
            None => (0, cow_str.as_ref(), false),
        };

        super::utils::assert_xid_chars(offset, ident)?;

        let inner = match cow_str {
            Cow::Borrowed(str) => {
                let str = match raw {
                    true => &str[RAW_START.len()..],
                    false => str,
                };

                Cow::Borrowed(str)
            }

            Cow::Owned(mut string) => {
                let string = match raw {
                    true => string.split_off(RAW_START.len()),
                    false => string,
                };

                Cow::Owned(string)
            }
        };

        Ok(AnyIdentifier { raw, inner })
    }
}

impl Display for AnyIdentifier<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.raw {
            f.write_str(RAW_START)?;
        }

        f.write_str(&self.inner)
    }
}

#[cfg(feature = "serde")]
mod serde {
    use alloc::string::String;

    use ::serde::{Deserialize, Serialize, Serializer};

    use super::*;

    impl<'a> Serialize for AnyIdentifier<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self.raw {
                true => {
                    let mut ident_string = String::with_capacity(RAW_START.len() + self.inner.len());
                    ident_string.push_str(RAW_START);
                    ident_string.push_str(&self.inner);
                    serializer.serialize_str(&ident_string)
                }
                false => serializer.serialize_str(&self.inner),
            }
        }
    }

    impl<'a, 'de: 'a> Deserialize<'de> for AnyIdentifier<'a> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            let cow_str = Cow::<'a, str>::deserialize(deserializer)?;
            Self::parse(cow_str).map_err(|err| ::serde::de::Error::custom(err.kind()))
        }
    }
}

#[cfg(feature = "syn")]
impl syn::parse::Parse for AnyIdentifier<'static> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use alloc::string::ToString;

        use syn::ext::IdentExt;

        let ident = input.call(syn::Ident::parse_any)?;

        // WORKAROUND: for syn::Ident not exposing `ident.is_raw()` and `ident.sym()`
        let Self { raw, inner } =
            AnyIdentifier::parse(ident.to_string()).expect("syn parse differs from internal parse");

        Ok(Self { raw, inner: Cow::Owned(inner.to_string()) })
    }
}

#[cfg(feature = "syn")]
impl<'a> From<AnyIdentifier<'a>> for syn::Ident {
    fn from(arg: AnyIdentifier<'a>) -> Self {
        match arg.raw() {
            true => syn::Ident::new_raw(&arg.inner, proc_macro2::Span::call_site()),
            false => syn::Ident::new(&arg.inner, proc_macro2::Span::call_site()),
        }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for AnyIdentifier<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let raw = self.raw;
        let inner = self.inner.as_ref();

        const DOC_MESSAGE: &str = "SAFETY: values provided from a validated `AnyIdentifier`";

        let identifier_tokens = quote::quote! {
            #[doc = #DOC_MESSAGE]
            unsafe { ::redefmt_args::identifier::AnyIdentifier::new_unchecked(#raw, #inner) }
        };

        tokens.extend(identifier_tokens);
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn parse() {
        let non_raw_result = AnyIdentifier::parse("x");
        assert!(non_raw_result.is_ok());

        let raw_result = AnyIdentifier::parse("r#x");
        assert!(raw_result.is_ok());
    }

    #[test]
    fn display() {
        assert_display("x");
        assert_display("r#x");

        fn assert_display(expected: &str) {
            let actual = AnyIdentifier::parse(expected).unwrap().to_string();
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn empty_error() {
        let expected = FormatStringParseError::new(0, 0..0, IdentifierParseError::Empty);
        let actual = AnyIdentifier::parse("").unwrap_err();
        assert_eq!(expected, actual);
    }

    #[test]
    fn serialization() {
        let non_raw_identifier = AnyIdentifier::parse("x").unwrap();
        serde_utils::assert::borrowed_bijective_serialization("\"x\"", &non_raw_identifier);

        let raw_identifier = AnyIdentifier::parse("r#x").unwrap();
        serde_utils::assert::borrowed_bijective_serialization("\"r#x\"", &raw_identifier);
    }

    #[test]
    fn to_tokens() {
        let inner = "r#x";

        let input = AnyIdentifier::parse(inner).unwrap();

        let expected = quote::quote! {
            #[doc = "SAFETY: values provided from a validated `AnyIdentifier`"]
            unsafe { ::redefmt_args::identifier::AnyIdentifier::new_unchecked(true, "x") }
        };

        crate::quote_utils::assert_tokens(input, expected);
    }
}
