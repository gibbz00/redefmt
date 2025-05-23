/// <https://doc.rust-lang.org/std/fmt/index.html#fillalignment>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormatAlign {
    pub alignment: Alignment,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub character: Option<char>,
}

impl FormatAlign {
    pub(crate) const fn new(alignment: Alignment, character: Option<char>) -> Self {
        Self { alignment, character }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for FormatAlign {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let alignment = &self.alignment;
        let character = crate::quote_utils::PrintOption::new(&self.character);

        let format_align_tokens = quote::quote! {
            ::redefmt_args::format_options::FormatAlign {
                alignment: #alignment,
                character: #character,
            }
        };

        tokens.extend(format_align_tokens);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Alignment {
    /// '<'
    Left,
    /// '^'
    Center,
    /// '>'
    Right,
}

impl Alignment {
    pub(crate) fn from_char(ch: char) -> Option<Self> {
        match ch {
            '<' => Some(Self::Left),
            '^' => Some(Self::Center),
            '>' => Some(Self::Right),
            _ => None,
        }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for Alignment {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let alignment_tokens = match self {
            Alignment::Left => quote! { ::redefmt_args::format_options::Alignment::Left },
            Alignment::Center => quote! { ::redefmt_args::format_options::Alignment::Center },
            Alignment::Right => quote! { ::redefmt_args::format_options::Alignment::Right },
        };

        tokens.extend(alignment_tokens);
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn to_tokens() {
        let input = FormatAlign { alignment: Alignment::Left, character: Some('a') };

        let expected = quote! {
            ::redefmt_args::format_options::FormatAlign {
                alignment: ::redefmt_args::format_options::Alignment::Left,
                character: Some('a'),
            }
        };

        crate::quote_utils::assert_tokens(input, expected);
    }
}
