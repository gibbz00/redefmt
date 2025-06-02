use crate::*;

/// Configuration for [`FormatProcessor::process_dynamic`]
pub struct DynamicProcessorConfig {
    /// Defaults to false to provide compatibility with Rust's `format_args!` error handling.
    ///
    /// Can for example be useful to disable if an library provided a collection
    /// arguments, but where the library user is only interested in printing
    /// some.
    pub disable_unused_named_check: bool,
    /// See [`Self::disable_unused_named_check`]
    pub disable_unused_positional_check: bool,
}

/// Configuration for [`FormatProcessor::process_static`]
#[impl_tools::autoimpl(Default)]
pub struct StaticProcessorConfig<C: ArgCapturer> {
    /// Configure a format argument capturer in order to provide functional
    /// parity with Rust's [format argument capturing][format_args_capture].
    ///
    /// Is some, then any named argument found in format string --but not in provided-- is pushed
    /// into it's capturable form. Meaning: `format!("{x} {match}")` becomes effectively expanded to
    /// `format!("{x} {match}", x = &x, match = r#match)`.
    ///
    /// If no argument capturer is configured, then an [`FormatProcessorError::MissingNamed`]
    /// is returned when a named argument in the format string is not found in the list of
    /// provided named arguments.
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
    /// Opt out of value compaction
    /// # Illustrative example if enabled:
    ///
    /// Compaction is done in three steps:
    ///
    /// ## Named Argument Compaction
    ///
    /// ```rust
    /// let a = 10;
    /// let before = format!("{x} {y}", x = a, y = a);
    /// let after = format!("{x} {x}", x = a);
    /// assert_eq!(before, after);
    /// ```
    ///
    /// ## Positional Argument Compaction
    ///
    /// ```rust
    /// let before = format!("{0} {1} {2} {3}", 1, 2, 1, 2);
    /// let after = format!("{0} {1} {0} {1}", 1, 2);
    /// assert_eq!(before, after);
    /// ```
    ///
    /// ## Positional with Named Argument Compaction
    ///
    /// ```rust
    /// let before = format!("{0} {x}", 1, x = 1);
    /// let after = format!("{0} {0}", 1);
    /// assert_eq!(before, after);
    /// ```
    pub disable_compaction: bool,
}
