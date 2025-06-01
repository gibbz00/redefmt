use crate::*;

/// Configure the argument resolver when constructing [`FormatExpression`]
#[impl_tools::autoimpl(Default)]
pub struct ResolverConfig<C: ArgCapturer> {
    /// Configure a format argument capturer in order to provide functional
    /// parity with Rust's [format argument capturing][format_args_capture].
    ///
    /// Is some, then any named argument found in format string --but not in provided-- is pushed
    /// into it's capturable form. Meaning: `format!("{x} {match}")` becomes effectively expanded to
    /// `format!("{x} {match}", x = &x, match = r#match)`.
    ///
    /// If no argument capturer is configured, then an [`ResolveArgsError::MissingNamed`] is
    /// returned when a named argument in the format string is not found in the list of provided
    /// named arguments.
    ///
    /// [format_args_capture]: https://rust-lang.github.io/rfcs/2795-format-args-implicit-identifiers.html
    pub arg_capturer: Option<C>,
    /// Defaults to false to provide compatibility with Rust's `format_args!` error handling.
    ///
    /// Can for example be useful to disable if an library provided a collection
    /// arguments, but where the library user is only interested in printing
    /// some.
    pub disable_unused_named_check: bool,
    /// See [`Self::disable_unused_named_check`]
    pub disable_unused_positional_check: bool,
}
