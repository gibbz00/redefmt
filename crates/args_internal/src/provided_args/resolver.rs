use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use hashbrown::HashSet;

use crate::*;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ResolveArgsError {
    #[error("invalid positional argument {0}, provided {1}, positional argument are zero-based")]
    InvalidStringPositional(usize, usize),
    #[error("provided {0} unused positional arguments")]
    UnusedPositionals(usize),
    #[error("provided named argument {0} not used in format string")]
    UnusedNamed(String),
}

pub struct ArgumentResolver<'a, 'aa> {
    format_string_args: Vec<&'aa mut FormatArgument<'a>>,
    provided_args: &'aa mut ProvidedArgs<'a>,
}

impl<'a, 'aa> ArgumentResolver<'a, 'aa> {
    /// Combine provided args with those in [`FormatString`]
    ///
    /// Performs a variety of context dependant checks, optimizations and
    /// disambiguations on both argument sources. Some provide functional
    /// parity with Rust's `format_args!`--notably [format argument
    /// capturing][format_args_capture]--, whilst others improve the efficiency of
    /// `redefmt`'s codec and printing.
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
    pub(crate) fn resolve(
        format_string: &'aa mut FormatString<'a>,
        provided_args: &'aa mut ProvidedArgs<'a>,
    ) -> Result<(), ResolveArgsError> {
        let this = Self { format_string_args: format_string.collect_args_mut(), provided_args };

        // Only valid as long as no provided args are removed. Could technically
        // reference provided args by temporarily having the args in an append
        // only structure--ex. from the elsa crate--for the duration it is
        // needed. Probably a bit overkill though.
        let provided_named_str_set = this.collect_named_set();

        this
            // preparation and validation
            .capture_and_validate_format_arguments(&provided_named_str_set)?
            .check_unused_provided_named(&provided_named_str_set)?
            .check_unused_provided_positionals()?
            // needs to be done before any merging as it effectively normalizes identifiers
            .unmove_idents()
            // deduplication steps
            .merge_named()
            .merge_positional()
            .reuse_named_in_positional();

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

    fn capture_and_validate_format_arguments(
        self,
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
                    // capture missing arguments
                    if !provided_named_str_set.contains(identifier) {
                        let captured_name = identifier.clone().into_any();

                        let captured_identifier = identifier.clone().into_safe_any();

                        let captured_variable = reference_ident(captured_identifier.into());

                        provided_args.named.push((captured_name, captured_variable));
                    };
                }
            }
        }

        Ok(Self { format_string_args, provided_args })
    }

    // This compiles:
    //
    // ```rust
    // let x = "x".to_string();
    // print!("{} {x}", x, x = x);
    // drop(x);
    // ```
    //
    // Because the print statement i transformed into: `print!("{} {x}", &x, x = &x);`
    fn unmove_idents(self) -> Self {
        self.provided_args.positional.iter_mut().for_each(unmove_expr);

        self.provided_args
            .named
            .iter_mut()
            .for_each(|(_, expr)| unmove_expr(expr));

        return self;

        fn unmove_expr(expr: &mut syn::Expr) {
            if let syn::Expr::Path(path) = expr
                && let Some(ident) = path.path.get_ident().cloned()
            {
                *expr = reference_ident(ident);
            }
        }
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

    // ```rust
    // let a = "x";
    // let before = format!("{0} {1} {2} {3}", 1, a, 1, a);
    // let after = format!("{0} {1} {0} {1}", 1, a);
    // assert_eq!(before, after);
    // ```
    fn merge_positional(self) -> Self {
        let Self { mut format_string_args, provided_args } = self;

        for (i, j) in Self::reversed_combinations(provided_args.positional.len()) {
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

    // ```rust
    // let a = 10;
    // let before = format!("{x} {y}", x = a, y = a);
    // let after = format!("{x} {x}", x = a);
    // assert_eq!(before, after);
    // ```
    fn merge_named(self) -> Self {
        let Self { mut format_string_args, provided_args } = self;

        let combinations = Self::reversed_combinations(provided_args.named.len());

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

    // ```rust
    // let before = format!("{0} {x}", 1, x = 1);
    // let after = format!("{0} {0}", 1);
    // assert_eq!(before, after);
    // ```
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

fn reference_ident(ident: syn::Ident) -> syn::Expr {
    syn::Expr::Reference(syn::ExprReference {
        attrs: Default::default(),
        and_token: Default::default(),
        mutability: None,
        expr: Box::new(syn::Expr::Path(syn::ExprPath {
            attrs: Default::default(),
            qself: None,
            path: syn::Path::from(ident),
        })),
    })
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn reversed_combinations() {
        let expected = alloc::vec![(2, 1), (2, 0), (1, 0)];
        let actual = ArgumentResolver::reversed_combinations(3).collect::<Vec<_>>();
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

    fn assert_resolve_unchanged(format_str: &'static str, provided_args: ProvidedArgs) {
        assert_resolve(format_str, provided_args.clone(), format_str, provided_args);
    }

    fn assert_resolve_unchanged_str(
        format_str: &'static str,
        provided_args: ProvidedArgs,
        provided_args_result: ProvidedArgs,
    ) {
        assert_resolve(format_str, provided_args.clone(), format_str, provided_args_result);
    }

    fn assert_resolve_unchanged_provided(
        format_str: &'static str,
        provided_args: ProvidedArgs,
        format_str_result: &str,
    ) {
        assert_resolve(format_str, provided_args.clone(), format_str_result, provided_args);
    }

    fn assert_resolve(
        format_str: &'static str,
        mut provided_args: ProvidedArgs,
        expected_format_str: &str,
        expected_provided_args: ProvidedArgs,
    ) {
        let mut format_string = FormatString::parse(format_str).unwrap();
        let expected_format_string = FormatString::parse(expected_format_str).unwrap();

        ArgumentResolver::resolve(&mut format_string, &mut provided_args).unwrap();

        assert_eq!(expected_format_string, format_string);
        assert_eq!(expected_provided_args, provided_args);
    }

    fn assert_resolve_err(format_str: &str, provided_args: ProvidedArgs, expected_error: ResolveArgsError) {
        let format_string = FormatString::parse(format_str).unwrap();

        let actual_error = FormatExpression::new(format_string, provided_args).unwrap_err();

        assert_eq!(expected_error, actual_error);
    }
}
