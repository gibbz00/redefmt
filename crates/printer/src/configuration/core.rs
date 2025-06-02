use redefmt_args::{
    format_string::{FormatString, FormatStringParseErrorKind},
    identifier::AnyIdentifier,
    processor::{DynamicProcessorConfig, FormatProcessor, FormatProcessorError, ProcessedFormatString},
};

use crate::*;

const EXPECTED_NAMED_FORMAT_ARGUMENTS: [AnyIdentifier<'static>; 6] =
    // SAFETY: all strings are valid identifiers
    unsafe {
        [
            AnyIdentifier::new_unchecked(false, "stamp"),
            AnyIdentifier::new_unchecked(false, "level"),
            AnyIdentifier::new_unchecked(false, "crate"),
            AnyIdentifier::new_unchecked(false, "file"),
            AnyIdentifier::new_unchecked(false, "line"),
            AnyIdentifier::new_unchecked(false, "statement"),
        ]
    };

const FORMAT_PROCESSOR_CONFIG: DynamicProcessorConfig = DynamicProcessorConfig {
    disable_unused_named_check: true,
    disable_unused_positional_check: false,
};

const DEFAULT_LOG_FORMAT_STRING: &str = "{stamp} [{level}] - {crate}: {statement}";

#[derive(Debug, thiserror::Error)]
pub enum PrettyPrinterConfigError {
    #[error("failed to parse format string")]
    FormatStringParse(#[from] FormatStringParseErrorKind),
    #[error("failed to process format string arguments")]
    ProcessedFormatString(#[from] FormatProcessorError),
}

pub struct PrettyPrinterConfig {
    pub(crate) stamp: PrintStampConfig,
    pub(crate) log_format_string: ProcessedFormatString<'static>,
}

impl PrettyPrinterConfig {
    /// Construct a new pretty printer configuration
    ///
    /// `log_format_str` parameter takes an optional format string which is provided the
    /// following named parameters, in order; "stamp", "level", "crate", "file", "line",
    /// "statement". Defaults to `"{stamp} [{level}] - {crate}: {statement}"` if none if
    /// provided.
    pub fn new(stamp_config: PrintStampConfig, log_format_str: Option<&str>) -> Result<Self, PrettyPrinterConfigError> {
        let log_format_str = log_format_str.unwrap_or(DEFAULT_LOG_FORMAT_STRING);

        let log_format_string = FormatString::parse(log_format_str)
            .map_err(|error| error.into_kind())?
            .owned();

        let log_format_string = FormatProcessor::process_dynamic(
            log_format_string,
            0,
            &EXPECTED_NAMED_FORMAT_ARGUMENTS,
            &FORMAT_PROCESSOR_CONFIG,
        )?;

        Ok(Self { stamp: stamp_config, log_format_string })
    }
}
