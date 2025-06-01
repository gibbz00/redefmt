use alloc::{string::ToString, vec::Vec};

use hashbrown::HashSet;

use crate::*;

pub struct InternalArgumentResolver<'a, 'aa, E> {
    format_string_args: Vec<&'aa mut FormatArgument<'a>>,
    provided_args: &'aa mut ProvidedArgs<'a, E>,
}

impl<'a, 'aa, E: PartialEq> InternalArgumentResolver<'a, 'aa, E> {
    pub(crate) fn resolve<C: ArgCapturer<Expression = E>>(
        format_string: &'aa mut FormatString<'a>,
        provided_args: &'aa mut ProvidedArgs<'a, E>,
        resolver_config: &ResolverConfig<C>,
    ) -> Result<(), ResolveArgsError> {
        let mut resolver = Self { format_string_args: format_string.collect_args_mut(), provided_args };

        // Only valid as long as no provided args are removed. Could technically
        // reference provided args by temporarily having the args in an append
        // only structure--ex. from the elsa crate--for the duration it is
        // needed. Probably a bit overkill though.
        let provided_named_str_set = resolver.collect_named_set();

        resolver = resolver
            // preparation and validation
            .capture_and_validate_format_arguments(resolver_config, &provided_named_str_set)?;

        if !resolver_config.disable_unused_named_check {
            resolver = resolver.check_unused_provided_named(&provided_named_str_set)?;
        }

        if !resolver_config.disable_unused_positional_check {
            resolver = resolver.check_unused_provided_positionals()?;
        }

        // needs to be done before any compaction as it effectively normalizes identifiers
        resolver = resolver.unmove_idents(resolver_config);

        if !resolver_config.disable_compaction {
            resolver.merge_named().merge_positional().reuse_named_in_positional();
        }

        Ok(())
    }

    fn collect_named_set(&self) -> HashSet<ArgumentIdentifier<'static>> {
        self.provided_args
            .named
            .iter()
            // idents unrawed since checks with format string arguments need to
            // match against both raw and non-raw alternatives
            .map(|(ident, _)| ident.clone().unraw().owned())
            .collect::<HashSet<_>>()
    }

    fn capture_and_validate_format_arguments<C: ArgCapturer<Expression = E>>(
        self,
        resolver_config: &ResolverConfig<C>,
        provided_named_str_set: &HashSet<ArgumentIdentifier>,
    ) -> Result<Self, ResolveArgsError> {
        let Self { mut format_string_args, provided_args } = self;

        for format_string_arg in format_string_args.iter_mut() {
            match &format_string_arg {
                FormatArgument::Index(argument_index) => {
                    let positional_len = provided_args.positional.len();

                    // check argument index within bounds and disambiguate indexed with named
                    if *argument_index >= positional_len {
                        let named_index = argument_index - positional_len;

                        let Some((arg_name, _)) = provided_args.named.get(named_index) else {
                            return Err(ResolveArgsError::InvalidStringPositional(
                                *argument_index,
                                positional_len + provided_args.named.len(),
                            ));
                        };

                        **format_string_arg = FormatArgument::Identifier(arg_name.clone().unraw());
                    }
                }
                FormatArgument::Identifier(identifier) => {
                    if !provided_named_str_set.contains(identifier) {
                        if let Some(arg_capturer) = &resolver_config.arg_capturer {
                            let captured_name = identifier.clone().into_any();

                            let captured_identifier = identifier.clone().into_safe_any();

                            let captured_variable = arg_capturer.transform_identifier(captured_identifier);

                            provided_args.named.push((captured_name, captured_variable));
                        } else {
                            return Err(ResolveArgsError::MissingNamed(identifier.to_string()));
                        }
                    };
                }
            }
        }

        Ok(Self { format_string_args, provided_args })
    }

    fn unmove_idents<C: ArgCapturer<Expression = E>>(self, resolver_config: &ResolverConfig<C>) -> Self {
        if let Some(arg_capturer) = &resolver_config.arg_capturer {
            self.provided_args.positional.iter_mut().for_each(|expr| {
                arg_capturer.unmove_expression(expr);
            });

            self.provided_args
                .named
                .iter_mut()
                .for_each(|(_, expr)| arg_capturer.unmove_expression(expr));
        }

        self
    }

    fn check_unused_provided_positionals(self) -> Result<Self, ResolveArgsError> {
        let Self { format_string_args, provided_args } = self;

        let format_string_positional_count = format_string_args
            .iter()
            .filter(|arg| matches!(arg, FormatArgument::Index(_)))
            .count();

        let provided_positional_count = provided_args.positional.len();

        if provided_positional_count > format_string_positional_count {
            return Err(ResolveArgsError::UnusedPositionals(
                provided_positional_count - format_string_positional_count,
            ));
        }

        Ok(Self { format_string_args, provided_args })
    }

    fn check_unused_provided_named(
        self,
        provided_named_str_set: &HashSet<ArgumentIdentifier>,
    ) -> Result<Self, ResolveArgsError> {
        let Self { format_string_args, provided_args } = self;

        let format_args_named_set = format_string_args
            .iter()
            .filter_map(|arg| match arg {
                FormatArgument::Identifier(identifier) => Some(identifier),
                FormatArgument::Index(_) => None,
            })
            .collect::<HashSet<_>>();

        for provided_named_str in provided_named_str_set {
            if !format_args_named_set.contains(provided_named_str) {
                return Err(ResolveArgsError::UnusedNamed(provided_named_str.to_string()));
            }
        }

        drop(format_args_named_set);

        Ok(Self { format_string_args, provided_args })
    }

    fn merge_positional(self) -> Self {
        let Self { mut format_string_args, provided_args } = self;

        for (i, j) in reversed_combinations(provided_args.positional.len()) {
            // None if element in index `i` was removed in a previous iteration
            if let Some(current_positional) = provided_args.positional.get(i) {
                let other_positional = &provided_args.positional[j];

                if current_positional == other_positional {
                    provided_args.positional.swap_remove(i);

                    for format_string_arg in format_string_args.iter_mut() {
                        if format_string_arg.matches_index(i) {
                            **format_string_arg = FormatArgument::Index(j);
                        }
                    }
                }
            }
        }

        Self { format_string_args, provided_args }
    }

    fn merge_named(self) -> Self {
        let Self { mut format_string_args, provided_args } = self;

        let combinations = reversed_combinations(provided_args.named.len());

        for (i, j) in combinations {
            // None if i was removed in a previous iteration, less performant
            // but simpler than a "skip while" approach. Skeptical to it
            // affecting affecting performance in any noticeable way.
            if let Some((current_name, current_value)) = provided_args.named.get(i) {
                let (other_name, other_value) = provided_args
                    .named
                    .get(j)
                    .expect("out of bounds indexes, `j` not less than `i`");

                if current_value == other_value {
                    let replacing_name = other_name.clone().unraw();
                    let matching_name = current_name.clone().unraw();

                    provided_args.named.swap_remove(i);

                    for format_string_arg in format_string_args.iter_mut() {
                        if let FormatArgument::Identifier(format_string_identifier) = format_string_arg
                            && format_string_identifier == &matching_name
                        {
                            *format_string_identifier = replacing_name.clone();
                        }
                    }
                }
            }
        }

        Self { format_string_args, provided_args }
    }

    fn reuse_named_in_positional(self) -> Self {
        let Self { mut format_string_args, provided_args } = self;
        for positional_index in (0..provided_args.positional.len()).rev() {
            // Values which appear again in other named arguments are merged
            // in `merge_value`.
            let maybe_name = provided_args.named.iter().find_map(|(name, named_arg)| {
                (named_arg == &provided_args.positional[positional_index]).then_some(name)
            });

            if let Some(name) = maybe_name {
                provided_args.positional.swap_remove(positional_index);

                let replacing_ident = name.clone().unraw();

                for format_string_arg in format_string_args.iter_mut() {
                    if format_string_arg.matches_index(positional_index) {
                        **format_string_arg = FormatArgument::Identifier(replacing_ident.clone());
                    }
                }
            }
        }

        Self { format_string_args, provided_args }
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

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn reversed_combinations() {
        let expected = alloc::vec![(2, 1), (2, 0), (1, 0)];
        let actual = super::reversed_combinations(3).collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn empty() {
        assert_resolve_unchanged("", parse_quote!());
    }

    #[test]
    fn basic_named() {
        assert_resolve_unchanged("{x}", parse_quote!(x = 10));
    }

    #[test]
    fn basic_indexed() {
        assert_resolve_unchanged("{0}", parse_quote!(10));
    }

    #[test]
    fn basic_raw() {
        assert_resolve_unchanged("{match}", parse_quote!(r#match = 10));
    }

    #[test]
    fn basic_keyword() {
        assert_resolve_unchanged("{match}", parse_quote!(match = 10));
    }

    #[test]
    fn basic_width() {
        assert_resolve_unchanged("{0:1$}", parse_quote!(10, 20));
    }

    #[test]
    fn basic_precision() {
        assert_resolve_unchanged("{0:.1$}", parse_quote!(10, 20));
    }

    #[test]
    fn next_precision_argument() {
        // if no spec, then first arg is precision, and second value
        assert_resolve_unchanged_provided("{:.*}", parse_quote!(1, 2), "{1:.0$}");

        // uses and increments counter for next unnamed arg
        assert_resolve_unchanged_provided("{} {:.*} {}", parse_quote!(1, 2, 3, 4), "{0} {2:.1$} {3}");
        assert_resolve_unchanged_provided("{} {0:.*} {}", parse_quote!(1, 2, 3), "{0} {0:.1$} {2}");

        // disambiguated into named
        assert_resolve_unchanged_provided("{x:.*}", parse_quote!(x = 3), "{x:.x$}");
        assert_resolve_unchanged_provided("{x:.*}", parse_quote!(y = 3, x = 10), "{x:.y$}");
    }

    #[test]
    fn capture_non_keyword() {
        // NOTE: captures by reference
        assert_resolve_unchanged_str("{x}", parse_quote!(), parse_quote!(x = &x));
    }

    #[test]
    fn capture_keyword() {
        assert_resolve_unchanged_str("{match}", parse_quote!(), parse_quote!(match = &r#match));
    }

    #[test]
    fn captures_width() {
        assert_resolve_unchanged_str("{0:x$}", parse_quote!(1), parse_quote!(1, x = &x));
    }

    #[test]
    fn captures_precision() {
        assert_resolve_unchanged_str("{0:.x$}", parse_quote!(1), parse_quote!(1, x = &x));
    }

    #[test]
    fn format_argument_unnamed_disambiguation() {
        assert_resolve_unchanged_provided("{} {0}", parse_quote!(10), "{0} {0}");
        assert_resolve_unchanged_provided("{0} {} {}", parse_quote!(10, 20), "{0} {0} {1}");
        assert_resolve_unchanged_provided("{} {} {0}", parse_quote!(10, 20), "{0} {1} {0}");
        assert_resolve_unchanged_provided("{} {1} {2} {}", parse_quote!(10, 20, 30), "{0} {1} {2} {1}");
    }

    #[test]
    fn format_argument_indexed_disambiguation() {
        assert_resolve_unchanged_provided("{0}", parse_quote!(x = 10), "{x}");
    }

    #[test]
    fn format_argument_reuse_named_in_positional() {
        assert_resolve_unchanged_provided("{}", parse_quote!(a = 10), "{a}");

        // index value and not format string argument position which
        // determines provided argument matching
        assert_resolve_unchanged_provided("{1} {a} {}", parse_quote!("x", a = 10), "{a} {a} {0}");

        // positional pointing to positional is unchanged
        assert_resolve_unchanged("{0} {a}", parse_quote!("x", a = 10));
    }

    #[test]
    fn format_argument_reuse_named_in_positional_is_derawed() {
        assert_resolve_unchanged_provided("{0}", parse_quote!(r#match = 10), "{match}");
    }

    #[test]
    fn format_argument_reuse_multiple_named_in_multiple_positional() {
        assert_resolve(
            "{} {}",
            parse_quote!(x = &a, y = 10),
            "{x} {y}",
            parse_quote!(x = &a, y = 10),
        );
    }

    #[test]
    fn unmove_positional_idents() {
        assert_resolve_unchanged_str("{0}", parse_quote!(x), parse_quote!(&x));
    }

    #[test]
    fn unmove_named_idents() {
        assert_resolve_unchanged_str("{x}", parse_quote!(x = y), parse_quote!(x = &y));
    }

    #[test]
    fn merge_named() {
        // any expression
        assert_resolve(
            "{x} {y}",
            parse_quote!(x = (1 + 2), y = (1 + 2)),
            "{x} {x}",
            parse_quote!(x = (1 + 2)),
        );

        // changes unrawed in format string
        assert_resolve(
            "{x} {y}",
            parse_quote!(r#x = 1, r#y = 1),
            "{x} {x}",
            parse_quote!(r#x = 1),
        );
    }

    #[test]
    fn inlining() {
        assert_resolve("{x} {y}", parse_quote!(x = x, y = x), "{x} {x}", parse_quote!(x = &x));
        // by ref
        assert_resolve("{x} {y}", parse_quote!(x = &x, y = &x), "{x} {x}", parse_quote!(x = &x));
        // mixed ref
        assert_resolve("{x} {y}", parse_quote!(x = x, y = &x), "{x} {x}", parse_quote!(x = &x));
    }

    #[test]
    fn merge_positional() {
        assert_resolve(
            "{0} {1} {2} {3}",
            parse_quote!(1, a, 1, a),
            "{0} {1} {0} {1}",
            parse_quote!(1, &a),
        );

        // applies to many
        assert_resolve("{1} {0} {1}", parse_quote!(1, 1), "{0} {0} {0}", parse_quote!(1));
    }

    #[test]
    fn reuse_named_in_positional() {
        // variables
        assert_resolve("{0} {x}", parse_quote!(a, x = a), "{x} {x}", parse_quote!(x = &a));
        // literals
        assert_resolve("{0} {x}", parse_quote!(1, x = 1), "{x} {x}", parse_quote!(x = 1));
        // repeatedly applied
        assert_resolve(
            "{0} {x} {1}",
            parse_quote!(1, 1, x = 1),
            "{x} {x} {x}",
            parse_quote!(x = 1),
        );
    }

    #[test]
    fn unused_named_error() {
        // both named with the same value asserts that the check is done before any deduplication
        assert_resolve_err(
            "{x}",
            parse_quote!(x = 10, y = 10),
            ResolveArgsError::UnusedNamed("y".to_string()),
        );
    }

    #[test]
    fn unused_positional_error() {
        assert_resolve_err("", parse_quote!(1), ResolveArgsError::UnusedPositionals(1));
        // both positionals with the same value asserts that the check is done before any deduplication
        assert_resolve_err("{}", parse_quote!(1, 1, "x"), ResolveArgsError::UnusedPositionals(2));

        // discontinuous indexes in format arguments are also captured
        assert_resolve_err(
            "{0} {2}",
            parse_quote!(1, 2, x = 3),
            ResolveArgsError::UnusedPositionals(1),
        );
    }

    #[test]
    fn invalid_positional_error() {
        assert_resolve_err("{1}", parse_quote!(), ResolveArgsError::InvalidStringPositional(1, 0));

        assert_resolve_err(
            "{0} {1}",
            parse_quote!(1),
            ResolveArgsError::InvalidStringPositional(1, 1),
        );

        // next argument usage requires when first is unnamed
        assert_resolve_err(
            "{:.*}",
            parse_quote!(1),
            ResolveArgsError::InvalidStringPositional(1, 1),
        );
    }

    fn assert_resolve_unchanged(format_str: &'static str, provided_args: ProvidedArgs<syn::Expr>) {
        assert_resolve(format_str, provided_args.clone(), format_str, provided_args);
    }

    fn assert_resolve_unchanged_str(
        format_str: &'static str,
        provided_args: ProvidedArgs<syn::Expr>,
        provided_args_result: ProvidedArgs<syn::Expr>,
    ) {
        assert_resolve(format_str, provided_args.clone(), format_str, provided_args_result);
    }

    fn assert_resolve_unchanged_provided(
        format_str: &'static str,
        provided_args: ProvidedArgs<syn::Expr>,
        format_str_result: &str,
    ) {
        assert_resolve(format_str, provided_args.clone(), format_str_result, provided_args);
    }

    fn assert_resolve(
        format_str: &'static str,
        mut provided_args: ProvidedArgs<syn::Expr>,
        expected_format_str: &str,
        expected_provided_args: ProvidedArgs<syn::Expr>,
    ) {
        let mut format_string = FormatString::parse(format_str).unwrap();
        let expected_format_string = FormatString::parse(expected_format_str).unwrap();

        let resolver_config = ResolverConfig { arg_capturer: Some(SynArgCapturer), ..Default::default() };
        InternalArgumentResolver::resolve(&mut format_string, &mut provided_args, &resolver_config).unwrap();

        assert_eq!(expected_format_string, format_string);
        assert_eq!(expected_provided_args, provided_args);
    }

    fn assert_resolve_err(format_str: &str, provided_args: ProvidedArgs<syn::Expr>, expected_error: ResolveArgsError) {
        let format_string = FormatString::parse(format_str).unwrap();

        let resolver_config = ResolverConfig { arg_capturer: Some(SynArgCapturer), ..Default::default() };
        let actual_error = FormatExpression::new(format_string, provided_args, &resolver_config).unwrap_err();

        assert_eq!(expected_error, actual_error);
    }
}
