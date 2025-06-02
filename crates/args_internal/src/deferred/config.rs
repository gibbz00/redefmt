/// Passed to [`ProcessedFormatString::format_deferred`](crate::ProcessedFormatString::format_deferred)
#[derive(Default, Clone, Copy)]
pub struct DeferredFormatConfig {
    /// Opt-out of `format_args!` "compatibility" by allowing format precision option arguments
    /// to be any signed or unsigned integer which can be successfully converted into a an `usize`.
    /// `usize` can normally be inferred for `format_args!`, but defaults to `i32` when
    /// deferred. Setting this to `true` prevents `deferred_format!("{n:.p$}", n = 1.0, p = 3)`
    /// from returning `DeferredFormatError::InvalidArgType` for `p`.
    pub allow_non_usize_precision_value: bool,
    /// Same as [`Self::allow_non_usize_precision_value`], but for format width options.
    pub allow_non_usize_width_value: bool,
}
