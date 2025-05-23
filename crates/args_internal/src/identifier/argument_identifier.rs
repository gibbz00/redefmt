use alloc::{borrow::Cow, string::ToString};
use core::fmt::{Debug, Display};

use check_keyword::CheckKeyword;

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
    pub unsafe fn new_unchecked(inner: Cow<'a, str>) -> Self {
        Self { inner }
    }

    pub(crate) fn owned(&self) -> ArgumentIdentifier<'static> {
        let ArgumentIdentifier { inner } = self;
        ArgumentIdentifier { inner: Cow::Owned(inner.to_string()) }
    }

    /// Makes `AnyIdentifier` raw is the identifier is a keyword
    pub(crate) fn into_safe_any(self) -> AnyIdentifier<'a> {
        AnyIdentifier { raw: self.is_keyword(), inner: self.inner }
    }

    pub(crate) fn into_any(self) -> AnyIdentifier<'a> {
        AnyIdentifier { raw: false, inner: self.inner }
    }

    pub(crate) fn parse(cow_str: impl Into<Cow<'a, str>>) -> Result<Self, FormatStringParseError> {
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
    fn empty_error() {
        let expected = FormatStringParseError::new(0, 0..0, IdentifierParseError::Empty);
        let actual = ArgumentIdentifier::parse("").unwrap_err();
        assert_eq!(expected, actual);
    }
}
