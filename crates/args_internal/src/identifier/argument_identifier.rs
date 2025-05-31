use alloc::{borrow::Cow, string::ToString};
use core::fmt::{Debug, Display};

use crate::*;

/// Non-raw identifier or keyword
///
/// Only suitable for format string argument identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArgumentIdentifier<'a> {
    pub(super) inner: Cow<'a, str>,
}

impl<'a> ArgumentIdentifier<'a> {
    #[doc(hidden)]
    pub const unsafe fn new_unchecked(inner: &'a str) -> Self {
        Self { inner: Cow::Borrowed(inner) }
    }

    pub(crate) fn owned(&self) -> ArgumentIdentifier<'static> {
        let ArgumentIdentifier { inner } = self;
        ArgumentIdentifier { inner: Cow::Owned(inner.to_string()) }
    }

    /// Makes `AnyIdentifier` raw is the identifier is a keyword
    #[cfg(feature = "syn")]
    pub(crate) fn into_safe_any(self) -> AnyIdentifier<'a> {
        use check_keyword::CheckKeyword;
        AnyIdentifier { raw: self.is_keyword(), inner: self.inner }
    }

    #[cfg(feature = "syn")]
    pub(crate) fn into_any(self) -> AnyIdentifier<'a> {
        AnyIdentifier { raw: false, inner: self.inner }
    }

    pub fn parse(cow_str: impl Into<Cow<'a, str>>) -> Result<Self, FormatStringParseError> {
        let cow_str = cow_str.into();

        if cow_str.is_empty() {
            return Err(FormatStringParseError::new(0, 0..0, IdentifierParseError::Empty));
        }

        Self::parse_impl(0, cow_str)
    }

    /// Context from `Argument::parse`:
    /// - `str` not empty
    pub(crate) fn parse_impl(offset: usize, cow_str: impl Into<Cow<'a, str>>) -> Result<Self, FormatStringParseError> {
        let cow_str = cow_str.into();

        if cow_str.starts_with(super::utils::RAW_START) {
            return Err(FormatStringParseError::new(
                offset,
                0..super::utils::RAW_START.len(),
                IdentifierParseError::RawIdent,
            ));
        }

        super::utils::assert_xid_chars(offset, &cow_str)?;

        Ok(ArgumentIdentifier { inner: cow_str })
    }
}

impl AsRef<str> for ArgumentIdentifier<'_> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Display for ArgumentIdentifier<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

#[cfg(feature = "serde")]
mod serde {
    use ::serde::{Deserialize, Serialize, Serializer};

    use super::*;

    impl<'a> Serialize for ArgumentIdentifier<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.inner)
        }
    }

    impl<'a, 'de: 'a> Deserialize<'de> for ArgumentIdentifier<'a> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            let cow_str = Cow::<'a, str>::deserialize(deserializer)?;
            Self::parse(cow_str).map_err(|err| ::serde::de::Error::custom(err.kind()))
        }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for ArgumentIdentifier<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let inner = self.inner.as_ref();

        const DOC_MESSAGE: &str = "SAFETY: provided inner str provided from a validated `ArgumentIdentifier`";

        let identifier_tokens = quote::quote! {
            #[doc = #DOC_MESSAGE]
            unsafe { ::redefmt_args::identifier::ArgumentIdentifier::new_unchecked(#inner) }
        };

        tokens.extend(identifier_tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let result = ArgumentIdentifier::parse("x");
        assert!(result.is_ok());
    }

    #[test]
    fn raw_identifier_error() {
        let expected_error = FormatStringParseError::new(0, 0..2, IdentifierParseError::RawIdent);
        let actual_error = ArgumentIdentifier::parse("r#x").unwrap_err();
        assert_eq!(expected_error, actual_error);
    }

    #[test]
    fn empty_error() {
        let expected = FormatStringParseError::new(0, 0..0, IdentifierParseError::Empty);
        let actual = ArgumentIdentifier::parse("").unwrap_err();
        assert_eq!(expected, actual);
    }

    #[test]
    fn display() {
        let expected = "x";
        let actual = ArgumentIdentifier::parse(expected).unwrap().to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn serialization() {
        let identifier = ArgumentIdentifier::parse("x").unwrap();
        serde_utils::assert::borrowed_bijective_serialization("\"x\"", &identifier);
    }

    #[test]
    fn to_tokens() {
        let inner = "x";

        let input = ArgumentIdentifier::parse(inner).unwrap();

        let expected = quote::quote! {
            #[doc = "SAFETY: provided inner str provided from a validated `ArgumentIdentifier`"]
            unsafe { ::redefmt_args::identifier::ArgumentIdentifier::new_unchecked(#inner) }
        };

        crate::quote_utils::assert_tokens(input, expected);
    }
}
