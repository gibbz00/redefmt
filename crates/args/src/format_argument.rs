use crate::*;

#[derive(Debug, Default, PartialEq)]
pub struct FormatArgument {
    identifier: Option<Argument>,
    options: FormatOptions,
}

impl Parse for FormatArgument {
    // Context:
    // - `str` is non-empty.
    // - Does not contain opening and closing braces.
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        // "{:}"
    }

    #[test]
    fn parse_with_argument_only() {
        // "{x:}"
        // "{x}"
    }

    #[test]
    fn parse_with_options_only() {
        // "{:?}"
    }

    #[test]
    fn parse_with_both_argument_and_options() {
        // "{x:?}"
    }
}
