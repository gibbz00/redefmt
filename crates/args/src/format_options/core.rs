use crate::*;

pub struct FormatOptions {
    align: Option<FormatAlign>,
    sign: Option<Sign>,
    use_alternate_form: bool,
    /// https://doc.rust-lang.org/std/fmt/index.html#sign0
    use_sign_aware_zero_padding: bool,
    width: Option<FormatWidth>,
    precision: Option<FormatPrecision>,
    format_trait: FormatTrait,
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
