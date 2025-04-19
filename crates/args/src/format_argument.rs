use crate::*;

pub struct FormatArgument {
    identifier: Option<Parameter>,
    options: FormatOptions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_format_option() {
        // "{:}"
    }
}
