use alloc::{borrow::Cow, string::ToString};

/// `Cow<str>` newtype encapsulating the invariant that all curly braces are escaped
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct FormatLiteral<'a>(Cow<'a, str>);

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum FormatLiteralError {
    #[error("unescaped opening brace, a second brace ('{{') must directly follow the first")]
    UnescapedOpen,
    #[error("unescaped closing brace, a second brace ('}}') must directly follow the first")]
    UnescapedClose,
}

impl<'a> FormatLiteral<'a> {
    #[doc(hidden)]
    pub const unsafe fn new_unchecked(str: &'a str) -> Self {
        Self(Cow::Borrowed(str))
    }

    pub fn new(cow_str: impl Into<Cow<'a, str>>) -> Result<Self, FormatLiteralError> {
        let cow_str = cow_str.into();

        let mut char_iter = cow_str.chars().peekable();

        while let Some(char) = char_iter.next() {
            if char == '{' {
                match char_iter.peek() == Some(&'{') {
                    true => char_iter.next(),
                    false => return Err(FormatLiteralError::UnescapedOpen),
                };
            }

            if char == '}' {
                match char_iter.peek() == Some(&'}') {
                    true => char_iter.next(),
                    false => return Err(FormatLiteralError::UnescapedClose),
                };
            }
        }

        Ok(Self(cow_str))
    }

    pub(crate) fn owned(&self) -> FormatLiteral<'static> {
        FormatLiteral(Cow::Owned(self.0.to_string()))
    }
}

impl AsRef<str> for FormatLiteral<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "serde")]
impl<'de: 'a, 'a> serde::Deserialize<'de> for FormatLiteral<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let cow_str = Cow::<str>::deserialize(deserializer)?;
        Self::new(cow_str).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for FormatLiteral<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let inner = self.0.as_ref();

        const DOC_MESSAGE: &str = "SAFETY: provided inner str provided from a validated `FormatLiteral`";

        let format_literal_tokens = quote::quote! {
            #[doc = #DOC_MESSAGE]
            unsafe { ::redefmt_args::format_literal::FormatLiteral::new_unchecked(#inner) }
        };

        tokens.extend(format_literal_tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ok() {
        let result = FormatLiteral::new("escaped open '{{' and escaped close '}}'");
        assert!(result.is_ok())
    }

    #[test]
    fn unescaped_open_error() {
        let actual = FormatLiteral::new("{").unwrap_err();
        let expected = FormatLiteralError::UnescapedOpen;
        assert_eq!(expected, actual);
    }

    #[test]
    fn unescaped_close_error() {
        let actual = FormatLiteral::new("}").unwrap_err();
        let expected = FormatLiteralError::UnescapedClose;
        assert_eq!(expected, actual);
    }

    #[test]
    fn serialization() {
        let json_str = "\"{{ abc }}\"";
        let input = FormatLiteral("{{ abc }}".into());
        crate::serde_utils::assert::borrowed_bijective_serialization(json_str, &input);
    }

    #[test]
    fn deserialize_errors() {
        let invalid_json_str = "\" { \"";
        let result = serde_json::from_str::<FormatLiteral>(invalid_json_str);
        assert!(result.is_err());
    }
}
