use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use hashbrown::HashSet;
use syn::{Token, ext::IdentExt, parse::Parse};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "provided-args-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvidedArgs {
    pub(crate) positional: Vec<ProvidedArgValue>,
    pub(crate) named: ProvidedNamedArgsMap,
}

impl ProvidedArgs {
    pub(crate) fn collect_named_set(&self) -> HashSet<String> {
        self.named
            .iter()
            // idents unrawed since checks with format string arguments need to
            // match against both raw and non-raw alternatives
            .map(|(ident, _)| ident.unraw().to_string())
            .collect::<HashSet<_>>()
    }
}

impl Parse for ProvidedArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut positional_args = Vec::new();
        let mut named_args = Vec::new();
        let mut registered_named_args = HashSet::new();

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

                if registered_named_args.contains(&name) {
                    return Err(syn::Error::new(name_span, "duplicate argument names not allowed"));
                } else {
                    named_args.push((name.clone(), value));
                    registered_named_args.insert(name);
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

        Ok(Self {
            positional: positional_args,
            named: ProvidedNamedArgsMap::from_inner(named_args),
        })
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn serialization() {
        let args = parse_quote!(1, "x", x = y, y = false);
        serde_utils::assert::bijective_serialization::<ProvidedArgs>(args);
    }

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

    fn mock_named() -> ProvidedNamedArgsMap {
        let inner = [
            (syn::Ident::new("x", Span::call_site()), parse_quote!(10)),
            (syn::Ident::new("match", Span::call_site()), parse_quote!(y)),
        ]
        .into_iter()
        .collect();

        ProvidedNamedArgsMap::from_inner(inner)
    }
}
