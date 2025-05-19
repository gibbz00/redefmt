use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use check_keyword::CheckKeyword;
use hashbrown::{HashMap, HashSet};
use syn::{Token, ext::IdentExt, parse::Parse};

use crate::*;

#[derive(Debug, PartialEq)]
pub struct ProvidedArgs {
    positional: Vec<ProvidedArgValue>,
    // NOTE: parsed with `syn::Ident::parse_any`
    named: HashMap<syn::Ident, ProvidedArgValue>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProvidedArgsError {
    #[error("format string positional argument count ({0}) does not match the provided count ({1})")]
    PositionalMismatch(usize, usize),
    #[error("provided named argument {0} not used in format string")]
    UnusedNamed(String),
}

impl ProvidedArgs {
    /// Consolidate provided arguments with [`FormatString::required_arguments`]
    ///
    /// Errors if there's a mismatch in the number of positional arguments, or if there's a named
    /// (unused) argument which is not present in required arguments.
    ///
    /// Named required arguments not present in `Args` are inserted into the
    /// named map with an `ArgValue` of the same name, effectively implementing
    /// [format argument capturing](format_args_capture).
    ///
    /// # Raw Identifiers and Argument Capturing
    ///
    /// Named format string arguments may be keywords, identifiers, but not raw
    /// identifiers. They can, however, capture their "unrawed" counterparts.
    /// The following are valid `format_args!` expressions:
    ///
    /// ```rust
    /// // Keyword as argument name
    /// format_args!("{match}", match = 10);
    ///
    /// // Raw idents for named arguments are "unrawed"
    /// format_args!("{match}", r#match = 10);
    ///
    /// // Raw ident in named argument value
    /// format_args!("{match}", match = r#match);
    ///
    /// // Raw ident as positional argument value
    /// let r#x = 20;
    /// format_args!("{}", r#x);
    ///
    /// // Captures raw ident into named format string argument
    /// let r#match = 10;
    /// format_args!("{match}");
    /// ```
    ///
    /// The last one is tricky because one can't simply add the add any missing
    /// arguments from the required args to the "provided" named arguments map.
    /// One needs to first check if the missing argument is a keyword, and if so
    /// capture the raw counterpart.
    ///
    /// [format_args_capture]: https://rust-lang.github.io/rfcs/2795-format-args-implicit-identifiers.html
    pub fn consolidate(&mut self, required_arguments: RequiredArgs) -> Result<(), ProvidedArgsError> {
        let required_named_arguments = required_arguments
            .named_arguments
            .iter()
            .map(|identifier| identifier.as_ref())
            .collect::<HashSet<_>>();

        let unrawed_provided = self
            .named
            .keys()
            .map(|ident| ident.unraw().to_string())
            .collect::<HashSet<_>>();

        {
            let required_count = required_arguments.unnamed_argument_count;
            let provided_count = self.positional.len();

            if required_count != provided_count {
                return Err(ProvidedArgsError::PositionalMismatch(required_count, provided_count));
            }
        }

        for provided_unrawed_name in &unrawed_provided {
            if !required_named_arguments.contains(provided_unrawed_name.as_str()) {
                return Err(ProvidedArgsError::UnusedNamed(provided_unrawed_name.clone()));
            }
        }

        for required_name in required_named_arguments {
            if !unrawed_provided.contains(required_name) {
                let captured_name = syn::parse_str(required_name).expect("invalid identifier");

                let captured_value = syn::parse_str(&required_name.into_safe())
                    .map(ProvidedArgValue::Variable)
                    .expect("invalid safe identifier");

                self.named.insert(captured_name, captured_value);
            }
        }

        Ok(())
    }
}

impl Parse for ProvidedArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut positional_args = Vec::new();
        let mut named_args = HashMap::new();

        loop {
            if input.is_empty() {
                break;
            }

            // named before positional so to not parse `x` in `x = 10` as a positional argument
            if input.peek2(Token![=]) {
                let name = input.call(syn::Ident::parse_any)?;
                let _ = input.parse::<Token![=]>()?;
                let value = input.parse()?;

                let name_span = name.span();

                if named_args.insert(name, value).is_some() {
                    return Err(syn::Error::new(name_span, "duplicate argument names not allowed"));
                }
            } else {
                let positional_arg: ProvidedArgValue = input.parse()?;

                if !named_args.is_empty() {
                    return Err(syn::Error::new(
                        positional_arg.span(),
                        "positional arguments can not follow named arguments",
                    ));
                } else {
                    positional_args.push(positional_arg);
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { positional: positional_args, named: named_args })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn parse_empty() {
        let expected = ProvidedArgs { positional: Default::default(), named: Default::default() };

        let actual = parse_quote!();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_positional() {
        let expected = ProvidedArgs { positional: mock_positional(), named: Default::default() };

        let actual = parse_quote!(10, a, "y");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_named() {
        let expected = ProvidedArgs { positional: Default::default(), named: mock_named() };

        let actual = parse_quote!(x = 10, match = y);

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_combined() {
        let expected = ProvidedArgs { positional: mock_positional(), named: mock_named() };

        let actual = parse_quote!(10, a, "y", x = 10, match = y);

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_with_trailing_comma() {
        let expected = ProvidedArgs { positional: Default::default(), named: mock_named() };

        let actual = parse_quote!(x = 10, match = y,);

        assert_eq!(expected, actual);
    }

    #[test]
    #[should_panic]
    fn positional_after_named_error() {
        let _: ProvidedArgs = parse_quote!(x = 10, "x");
    }

    #[test]
    #[should_panic]
    fn duplicate_name_error() {
        let _: ProvidedArgs = parse_quote!(x = 10, x = 20);
    }

    fn mock_positional() -> Vec<ProvidedArgValue> {
        alloc::vec![parse_quote!(10), parse_quote!(a), parse_quote!("y")]
    }

    fn mock_named() -> HashMap<syn::Ident, ProvidedArgValue> {
        HashMap::from_iter([
            (syn::parse_str("x").unwrap(), parse_quote!(10)),
            (syn::parse_str("match").unwrap(), parse_quote!(y)),
        ])
    }
}
