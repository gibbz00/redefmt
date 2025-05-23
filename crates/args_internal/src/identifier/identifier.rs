use alloc::{borrow::Cow, string::ToString};
use core::fmt::{Debug, Display};

use crate::*;

/// Non-raw identifier or keyword
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
    inner: Cow<'a, str>,
}

#[cfg(feature = "provided-args")]
impl Identifier<'static> {
    pub fn from_provided(ident: &syn::Ident) -> Self {
        use syn::ext::IdentExt;
        Self { inner: ident.unraw().to_string().into() }
    }
}

impl<'a> Identifier<'a> {
    pub(crate) fn owned(&self) -> Identifier<'static> {
        let Identifier { inner } = self;
        Identifier { inner: Cow::Owned(inner.to_string()) }
    }

    /// Context from `Argument::parse`:
    /// - `str` not empty
    pub(crate) fn parse(offset: usize, cow_str: impl Into<Cow<'a, str>>) -> Result<Self, ParseError> {
        let cow_str = cow_str.into();

        if cow_str.starts_with(super::utils::RAW_START) {
            return Err(ParseError::new(
                offset,
                0..super::utils::RAW_START.len(),
                IdentifierParseError::RawIdent,
            ));
        }

        super::utils::assert_xid_chars(offset, &cow_str)?;

        Ok(Identifier { inner: cow_str })
    }
}

impl AsRef<str> for Identifier<'_> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Display for Identifier<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

#[cfg(feature = "serde")]
mod serde {
    use ::serde::{Deserialize, Serialize, Serializer};

    use super::*;

    impl<'a> Serialize for Identifier<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.inner)
        }
    }

    impl<'a, 'de: 'a> Deserialize<'de> for Identifier<'a> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            let cow_str = Cow::<'a, str>::deserialize(deserializer)?;
            Self::parse(0, cow_str).map_err(|err| ::serde::de::Error::custom(err.kind()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let result = Identifier::parse(0, "x");
        assert!(result.is_ok());
    }

    #[test]
    fn raw_identifier_error() {
        let expected_error = ParseError::new(0, 0..2, IdentifierParseError::RawIdent);
        let actual_error = Identifier::parse(0, "r#x").unwrap_err();
        assert_eq!(expected_error, actual_error);
    }

    #[test]
    fn display() {
        let expected = "x";
        let actual = Identifier::parse(0, expected).unwrap().to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn serialization() {
        let identifier = Identifier::parse(0, "x").unwrap();
        serde_utils::assert::borrowed_bijective_serialization("\"x\"", &identifier);
    }
}
