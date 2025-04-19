use crate::*;

pub struct FormatArgument {
    identifier: Option<Argument>,
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
