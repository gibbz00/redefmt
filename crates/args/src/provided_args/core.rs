use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use check_keyword::CheckKeyword;
use hashbrown::HashSet;
use proc_macro2::Span;
use syn::{Token, ext::IdentExt, parse::Parse};

use crate::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "provided-args-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvidedArgs {
    positional: Vec<ProvidedArgValue>,
    named: ProvidedNamedArgsMap,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ProvidedArgsError {
    #[error("invalid positional argument {0}, provided {1}, positional argument are zero-based")]
    InvalidStringPositional(usize, usize),
    #[error("provided {0} unused positional arguments")]
    UnusedPositionals(usize),
    #[error("provided named argument {0} not used in format string")]
    UnusedNamed(String),
}

impl ProvidedArgs {
    /// Resolve provided with those in [`FormatString`]
    ///
    /// Performs a variety of context dependant checks, optimizations and
    /// disambiguations on both argument sources. Some provide functional
    /// parity with Rust's `format_args!`--notably [format argument
    /// capturing](format_args_capture)--, whilst others improve the efficiency of
    /// `redefmt`'s codec and printing.
    ///
    /// # Deduplication of Provided Values
    ///
    /// Matching positional and named values are depuplicated in four steps:
    ///
    /// ```rust
    /// // inline named arguments:
    /// let x = 1;
    /// let before = format!("{x} {y}", x = x, y = x);
    /// let after = format!("{x} {x}", x = x);
    /// assert_eq!(before, after);
    ///
    /// // merge positionals
    /// let a = "x";
    /// let before = format!("{0} {1} {2} {3}", 1, a, 1, a);
    /// let after = format!("{0} {1} {0} {1}", 1, a);
    /// assert_eq!(before, after);
    ///
    /// // merge named arguments:
    /// let a = 10;
    /// let before = format!("{x} {y}", x = a, y = a);
    /// let after = format!("{x} {x}", x = a);
    /// assert_eq!(before, after);
    ///
    /// // merge positional and named arguments:
    /// let before = format!("{0} {x}", 1, x = 1);
    /// let after = format!("{0} {0}", 1);
    /// assert_eq!(before, after);
    /// ```
    ///
    /// # Format String Argument Disambigaution
    ///
    /// Unnamed positional arguments are disambiguated to indexed positional
    /// arguments. If the index points to a named argument, then the positional
    /// argument is replaced with the corresponding named argument.
    ///
    /// ```rust
    /// let before = format!("{1} {}", 1, x = 2);
    /// let after = format!("{x} {0}", 1, x = 2);
    /// assert_eq!(before, after);
    /// ```
    ///
    /// [format_args_capture]: https://rust-lang.github.io/rfcs/2795-format-args-implicit-identifiers.html
    pub fn consolidate(&mut self, format_string: &mut FormatString) -> Result<(), ProvidedArgsError> {
        let mut format_string_args = format_string.collect_args_mut();

        let provided_named_str_set = self.collect_named_set();

        // preparation and validation
        self.capture_and_validate_format_arguments(&mut format_string_args, &provided_named_str_set)?;
        self.check_unused_provided_named(&format_string_args, &provided_named_str_set)?;
        self.check_unused_provided_positionals(&format_string_args)?;

        // deduplication steps
        self.inline_named(&mut format_string_args);
        self.merge_positional(&mut format_string_args);
        self.merge_named(&mut format_string_args);
        self.reuse_named_in_positional(&mut format_string_args);

        Ok(())
    }

    fn collect_named_set(&self) -> HashSet<String> {
        self.named
            .iter()
            // idents unrawed since checks with format string arguments need to
            // match against both raw and non-raw alternatives
            .map(|(ident, _)| ident.unraw().to_string())
            .collect::<HashSet<_>>()
    }

    fn capture_and_validate_format_arguments(
        &mut self,
        format_string_args: &mut [&mut FormatArgument],
        provided_named_str_set: &HashSet<String>,
    ) -> Result<(), ProvidedArgsError> {
        for format_string_arg in format_string_args.iter_mut() {
            match &format_string_arg {
                FormatArgument::Index(argument_index) => {
                    let argument_index = argument_index.inner();
                    let positional_len = self.positional.len();

                    // check argument index within bounds and disambiguate indexed with named
                    if argument_index >= positional_len {
                        let named_index = argument_index - positional_len;

                        let Some((arg_name, _)) = self.named.get(named_index) else {
                            return Err(ProvidedArgsError::InvalidStringPositional(
                                argument_index,
                                positional_len + self.named.len(),
                            ));
                        };

                        **format_string_arg = FormatArgument::Identifier(Identifier::from_provided(arg_name));
                    }
                }
                FormatArgument::Identifier(identifier) => {
                    // capture missing arguments
                    if !provided_named_str_set.contains(identifier.as_ref()) {
                        let captured_name = syn::Ident::new(identifier.as_ref(), Span::call_site());

                        let captured_variable = {
                            let ident_str = identifier.as_ref();

                            let call_site = Span::call_site();

                            let ident = match identifier.is_keyword() {
                                true => syn::Ident::new_raw(ident_str, call_site),
                                false => syn::Ident::new(ident_str, call_site),
                            };

                            ProvidedArgValue::Variable(ident)
                        };

                        self.named.push((captured_name, captured_variable));
                    };
                }
            }
        }

        Ok(())
    }

    fn check_unused_provided_positionals(
        &self,
        format_string_args: &[&mut FormatArgument],
    ) -> Result<(), ProvidedArgsError> {
        let format_string_positional_count = format_string_args
            .iter()
            .filter(|arg| matches!(arg, FormatArgument::Index(_)))
            .count();

        let provided_positional_count = self.positional.len();

        if provided_positional_count > format_string_positional_count {
            return Err(ProvidedArgsError::UnusedPositionals(
                provided_positional_count - format_string_positional_count,
            ));
        }

        Ok(())
    }

    fn check_unused_provided_named(
        &self,
        format_string_args: &[&mut FormatArgument],
        provided_named_str_set: &HashSet<String>,
    ) -> Result<(), ProvidedArgsError> {
        let format_args_named_set = format_string_args
            .iter()
            .filter_map(|arg| match arg {
                FormatArgument::Identifier(identifier) => Some(identifier.as_ref()),
                FormatArgument::Index(_) => None,
            })
            .collect::<HashSet<_>>();

        for provided_named_str in provided_named_str_set {
            if !format_args_named_set.contains(provided_named_str.as_str()) {
                return Err(ProvidedArgsError::UnusedNamed(provided_named_str.to_string()));
            }
        }

        Ok(())
    }

    fn inline_named(&mut self, format_string_args: &mut [&mut FormatArgument]) {
        for current_index in (0..self.named.len()).rev() {
            let (current_name, current_value) = self.named.get(current_index).expect("index out of bounds");
            if let ProvidedArgValue::Variable(variable_value) = current_value {
                let inlined = self
                    .named
                    .iter()
                    .any(|(key, value)| key == variable_value && value != current_value);

                if inlined {
                    let redundant_identifier = Identifier::from_provided(current_name);
                    let replacing_identifier = Identifier::from_provided(variable_value);

                    self.named.swap_remove(current_index);

                    for format_string_arg in format_string_args.iter_mut() {
                        if format_string_arg.matches_name(&redundant_identifier) {
                            **format_string_arg = FormatArgument::Identifier(replacing_identifier.clone())
                        }
                    }
                }
            }
        }
    }

    fn merge_positional(&mut self, format_string_args: &mut [&mut FormatArgument]) {
        for (i, j) in Self::reversed_combinations(self.positional.len()) {
            // None if element in index `i` was removed in a previous iteration
            if let Some(current_positional) = self.positional.get(i) {
                let other_positional = &self.positional[j];

                if current_positional == other_positional {
                    self.positional.swap_remove(i);

                    for format_string_arg in format_string_args.iter_mut() {
                        if format_string_arg.matches_index(i) {
                            **format_string_arg = FormatArgument::Index(Integer::new(j));
                        }
                    }
                }
            }
        }
    }

    fn merge_named(&mut self, format_string_args: &mut [&mut FormatArgument]) {
        let combinations = Self::reversed_combinations(self.named.len());

        for (i, j) in combinations {
            // None if i was removed in a previous iteration, less performant
            // but simpler than a "skip while" approach. Skeptical to it
            // affecting affecting performance in any noticeable way.
            if let Some((current_name, current_value)) = self.named.get(i) {
                let (other_name, other_value) =
                    self.named.get(j).expect("out of bounds indexes, `j` not less than `i`");

                if current_value == other_value {
                    let replacing_name = Identifier::from_provided(other_name);
                    let matching_name = Identifier::from_provided(current_name);

                    self.named.swap_remove(i);

                    for format_string_arg in format_string_args.iter_mut() {
                        if let FormatArgument::Identifier(format_string_identifier) = format_string_arg {
                            if format_string_identifier == &matching_name {
                                *format_string_identifier = replacing_name.clone();
                            }
                        }
                    }
                }
            }
        }
    }

    fn reuse_named_in_positional(&mut self, format_string_args: &mut [&mut FormatArgument]) {
        for positional_index in (0..self.positional.len()).rev() {
            // Values which appear again in other named arguments are merged
            // in `merge_value`.
            let maybe_name = self
                .named
                .iter()
                .find_map(|(name, named_arg)| (named_arg == &self.positional[positional_index]).then_some(name));

            if let Some(name) = maybe_name {
                self.positional.swap_remove(positional_index);

                let replacing_ident = Identifier::from_provided(name);

                for format_string_arg in format_string_args.iter_mut() {
                    if format_string_arg.matches_index(positional_index) {
                        **format_string_arg = FormatArgument::Identifier(replacing_ident.clone());
                    }
                }
            }
        }
    }

    /// Returns all unique index combinations for a collection of a given length.
    ///
    /// Indexes are flipped in such a way that i > j for any (i, j) item
    /// returned by the iterator. Morover, the iterator is reversed in such a
    /// way that i >= i_next. The intent of this is to make it possible simply remove a
    /// an element at index `i` during iteration.
    fn reversed_combinations(len: usize) -> impl Iterator<Item = (usize, usize)> {
        (0..len).flat_map(move |j| (j + 1..len).map(move |i| (i, j))).rev()
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
    use syn::parse_quote;

    use super::*;

    mod parse {
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

    mod consolidate {
        use alloc::string::ToString;

        use super::*;

        #[test]
        fn empty() {
            assert_consolidate_unchanged("", parse_quote!());
        }

        #[test]
        fn basic_named() {
            assert_consolidate_unchanged("{x}", parse_quote!(x = 10));
        }

        #[test]
        fn basic_indexed() {
            assert_consolidate_unchanged("{0}", parse_quote!(10));
        }

        #[test]
        fn basic_raw() {
            assert_consolidate_unchanged("{match}", parse_quote!(r#match = 10));
        }

        #[test]
        fn basic_keyword() {
            assert_consolidate_unchanged("{match}", parse_quote!(match = 10));
        }

        #[test]
        fn basic_width() {
            assert_consolidate_unchanged("{0:1$}", parse_quote!(10, 20));
        }

        #[test]
        fn basic_precision() {
            assert_consolidate_unchanged("{0:.1$}", parse_quote!(10, 20));
        }

        #[test]
        fn capture_non_keyword() {
            assert_consolidate_unchanged_str("{x}", parse_quote!(), parse_quote!(x = x));
        }

        #[test]
        fn capture_keyword() {
            assert_consolidate_unchanged_str("{match}", parse_quote!(), parse_quote!(match = r#match));
        }

        #[test]
        fn captures_width() {
            assert_consolidate_unchanged_str("{0:x$}", parse_quote!(1), parse_quote!(1, x = x));
        }

        #[test]
        fn captures_precision() {
            assert_consolidate_unchanged_str("{0:.x$}", parse_quote!(1), parse_quote!(1, x = x));
        }

        #[test]
        fn format_argument_unnamed_disambiguation() {
            assert_consolidate_unchanged_provided("{} {0}", parse_quote!(10), "{0} {0}");
            assert_consolidate_unchanged_provided("{0} {} {}", parse_quote!(10, 20), "{0} {0} {1}");
            assert_consolidate_unchanged_provided("{} {} {0}", parse_quote!(10, 20), "{0} {1} {0}");
            assert_consolidate_unchanged_provided("{} {1} {2} {}", parse_quote!(10, 20, 30), "{0} {1} {2} {1}");
        }

        #[test]
        fn format_argument_indexed_disambiguation() {
            assert_consolidate_unchanged_provided("{0}", parse_quote!(x = 10), "{x}");
        }

        #[test]
        fn format_argument_reuse_named_in_positional() {
            assert_consolidate_unchanged_provided("{}", parse_quote!(a = 10), "{a}");

            // index value and not format string argument position which
            // determines provided argument matching
            assert_consolidate_unchanged_provided("{1} {a} {}", parse_quote!("x", a = 10), "{a} {a} {0}");

            // positional pointing to positional is unchanged
            assert_consolidate_unchanged("{0} {a}", parse_quote!("x", a = 10));
        }

        #[test]
        fn format_argument_reuse_named_in_positional_is_derawed() {
            assert_consolidate_unchanged_provided("{0}", parse_quote!(r#match = 10), "{match}");
        }

        #[test]
        fn format_argument_reuse_multiple_named_in_multiple_positional() {
            assert_consolidate(
                "{} {}",
                parse_quote!(x = a, y = 10),
                "{x} {y}",
                parse_quote!(x = a, y = 10),
            );
        }

        #[test]
        fn merge_named() {
            // variables
            assert_consolidate("{x} {y}", parse_quote!(x = x, y = x), "{x} {x}", parse_quote!(x = x));
            // literals
            assert_consolidate("{x} {y}", parse_quote!(x = 1, y = 1), "{x} {x}", parse_quote!(x = 1));

            // changes unrawed in format string
            assert_consolidate(
                "{x} {y}",
                parse_quote!(r#x = 1, r#y = 1),
                "{x} {x}",
                parse_quote!(r#x = 1),
            );

            // repeatedly applied
            assert_consolidate(
                "{a} {x} {x} {y}",
                parse_quote!(x = y, y = a),
                "{a} {a} {a} {a}",
                parse_quote!(a = a),
            );
        }

        #[test]
        fn inline_named() {
            assert_consolidate("{x} {y}", parse_quote!(x = 1, y = x), "{x} {x}", parse_quote!(x = 1));
        }

        #[test]
        fn merge_positional() {
            assert_consolidate(
                "{0} {1} {2} {3}",
                parse_quote!(1, a, 1, a),
                "{0} {1} {0} {1}",
                parse_quote!(1, a),
            );

            // applies to many
            assert_consolidate("{1} {0} {1}", parse_quote!(1, 1), "{0} {0} {0}", parse_quote!(1));
        }

        #[test]
        fn reuse_named_in_positional() {
            // variables
            assert_consolidate("{0} {x}", parse_quote!(a, x = a), "{x} {x}", parse_quote!(x = a));
            // literals
            assert_consolidate("{0} {x}", parse_quote!(1, x = 1), "{x} {x}", parse_quote!(x = 1));
            // repeatedly applied
            assert_consolidate(
                "{0} {x} {1}",
                parse_quote!(1, 1, x = 1),
                "{x} {x} {x}",
                parse_quote!(x = 1),
            );
        }

        #[test]
        fn unused_named_error() {
            // both named with the same value asserts that the check is done before any deduplication
            assert_consolidate_err(
                "{x}",
                parse_quote!(x = 10, y = 10),
                ProvidedArgsError::UnusedNamed("y".to_string()),
            );
        }

        #[test]
        fn unused_positional_error() {
            assert_consolidate_err("", parse_quote!(1), ProvidedArgsError::UnusedPositionals(1));
            // both positionals with the same value asserts that the check is done before any deduplication
            assert_consolidate_err("{}", parse_quote!(1, 1, "x"), ProvidedArgsError::UnusedPositionals(2));
        }

        #[test]
        fn invalid_positional_error() {
            assert_consolidate_err("{1}", parse_quote!(), ProvidedArgsError::InvalidStringPositional(1, 0));

            assert_consolidate_err(
                "{0} {1}",
                parse_quote!(1),
                ProvidedArgsError::InvalidStringPositional(1, 1),
            );

            // discontinuous indexes in format arguments are also captured
            assert_consolidate_err(
                "{0} {2}",
                parse_quote!(1, 2, x = 3),
                ProvidedArgsError::UnusedPositionals(1),
            );
        }

        fn assert_consolidate_unchanged(format_str: &str, provided_args: ProvidedArgs) {
            assert_consolidate(format_str, provided_args.clone(), format_str, provided_args);
        }

        fn assert_consolidate_unchanged_str(
            format_str: &str,
            provided_args: ProvidedArgs,
            provided_args_result: ProvidedArgs,
        ) {
            assert_consolidate(format_str, provided_args.clone(), format_str, provided_args_result);
        }

        fn assert_consolidate_unchanged_provided(
            format_str: &str,
            provided_args: ProvidedArgs,
            format_str_result: &str,
        ) {
            assert_consolidate(format_str, provided_args.clone(), format_str_result, provided_args);
        }

        fn assert_consolidate(
            format_str: &str,
            mut provided_args: ProvidedArgs,
            expected_format_str: &str,
            expected_provided_args: ProvidedArgs,
        ) {
            let mut format_string = FormatString::parse(format_str).unwrap();
            let expected_format_string = FormatString::parse(expected_format_str).unwrap();

            provided_args.consolidate(&mut format_string).unwrap();

            assert_eq!(
                expected_format_string, format_string,
                "when expecting '{expected_format_str}' from '{format_str}'"
            );
            assert_eq!(expected_provided_args, provided_args);
        }

        fn assert_consolidate_err(format_str: &str, mut args: ProvidedArgs, expected_error: ProvidedArgsError) {
            let mut format_string = FormatString::parse(format_str).unwrap();

            let actual_error = args.consolidate(&mut format_string).unwrap_err();

            assert_eq!(expected_error, actual_error);
        }
    }

    mod utils {
        use super::*;

        #[test]
        fn reversed_combinations() {
            let expected = alloc::vec![(2, 1), (2, 0), (1, 0)];
            let actual = ProvidedArgs::reversed_combinations(3).collect::<Vec<_>>();
            assert_eq!(expected, actual);
        }
    }

    mod serde {
        use super::*;

        #[test]
        fn serialization() {
            let args = parse_quote!(1, "x", x = y, y = false);
            serde_utils::assert::bijective_serialization::<ProvidedArgs>(args);
        }
    }
}
