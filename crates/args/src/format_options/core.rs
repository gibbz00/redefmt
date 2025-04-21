use crate::*;

#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FormatOptions {
    align: Option<FormatAlign>,
    sign: Option<Sign>,
    #[cfg_attr(feature = "builder", builder(default))]
    use_alternate_form: bool,
    /// https://doc.rust-lang.org/std/fmt/index.html#sign0
    #[cfg_attr(feature = "builder", builder(default))]
    use_sign_aware_zero_padding: bool,
    width: Option<FormatWidth>,
    precision: Option<FormatPrecision>,
    #[cfg_attr(feature = "builder", builder(default))]
    format_trait: FormatTrait,
}

impl Parse for FormatOptions {
    /// Context from `FormatArgument::parse`:
    /// - Does not include semicolon.
    /// - `str` may be empty.
    /// - Contains no trailing whitespace.
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        if str.is_empty() {
            return Ok(Self::default());
        }

        // TEMP: placeholder for tests in FormatArgument
        if str == "?" {
            return Ok(Self { format_trait: FormatTrait::Debug, ..Default::default() });
        }

        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_option_whitespace() {
        // "{:? }"
        // "{: }"
        // "{:  }"
        // "{:\t\n}"
    }
}
