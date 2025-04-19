use crate::*;

#[derive(Debug, Default, PartialEq)]
pub struct FormatArgument {
    identifier: Option<Argument>,
    options: FormatOptions,
}

impl Parse for FormatArgument {
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        // "{}"
        // "{:}"
    }

    #[test]
    fn parse_with_argument_only() {
        // "{abc:}"
        // "{abc}"
    }

    #[test]
    fn parse_with_options_only() {
        // "{:?}"
    }
}
