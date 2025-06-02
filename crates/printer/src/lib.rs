//! # `redefmt-printer`

// TEMP:
#![allow(missing_docs)]

pub use printer::PrettyPrinter;
mod printer {
    use std::time::Duration;

    use chrono::{DateTime, Utc};
    use redefmt_args::{
        deferred::{
            DeferredFormatError, DeferredStructVariant, DeferredTypeValue, DeferredTypeVariant, DeferredValue,
            DeferredValues,
        },
        identifier::AnyIdentifier,
        processor::ProcessedFormatString,
    };
    use redefmt_decoder::{
        RedefmtFrame,
        values::{DecodedValues, StructVariantValue, TypeStructureValue, TypeStructureVariantValue, Value},
    };

    use crate::*;

    pub struct PrettyPrinter {
        first_frame_start: Option<DateTime<Utc>>,
        config: PrettyPrinterConfig,
    }

    impl PrettyPrinter {
        pub fn new(config: PrettyPrinterConfig) -> Self {
            Self { first_frame_start: None, config }
        }

        pub fn log_statement(&mut self, redefmt_frame: RedefmtFrame) -> Result<String, DeferredFormatError> {
            let RedefmtFrame {
                level,
                stamp,
                file_name,
                file_line,
                format_string,
                append_newline,
                decoded_values,
            } = redefmt_frame;

            let stamp = stamp.map(|stamp| self.evaluate_stamp(stamp)).unwrap_or_default();

            let statement = Self::evaluate_statement(format_string, &decoded_values, append_newline)?;

            // FIXME: crate_name from frame
            let crate_name = "temp_crate_name";

            // SAFETY: `AnyIdentifier`s constructed with are valid identifier strings
            let named_values = unsafe {
                [
                    (
                        AnyIdentifier::new_unchecked(false, "stamp"),
                        DeferredValue::String(stamp.into()),
                    ),
                    (
                        AnyIdentifier::new_unchecked(false, "level"),
                        DeferredValue::String(level.map(|level| level.to_string().into()).unwrap_or("NONE".into())),
                    ),
                    (
                        AnyIdentifier::new_unchecked(false, "crate"),
                        DeferredValue::String(crate_name.into()),
                    ),
                    (
                        AnyIdentifier::new_unchecked(false, "file"),
                        DeferredValue::String(file_name.into()),
                    ),
                    (
                        AnyIdentifier::new_unchecked(false, "line"),
                        DeferredValue::U32(file_line),
                    ),
                    (
                        AnyIdentifier::new_unchecked(false, "statement"),
                        DeferredValue::String(statement.into()),
                    ),
                ]
            };

            let deferred_values = DeferredValues::new([], named_values);

            self.config.log_format_string.format_deferred(&deferred_values)
        }

        fn evaluate_stamp(&mut self, stamp: u64) -> String {
            match &self.config.stamp {
                PrintStampConfig::Counter => stamp.to_string(),
                PrintStampConfig::OffsetTimestamp(timestamp_config) => {
                    let first_frame_start = *self.first_frame_start.get_or_insert_with(Utc::now);

                    let duration_offset = match timestamp_config.timestamp_precision {
                        PrintTimestampPrecisionConfig::Microseconds => Duration::from_micros(stamp),
                        PrintTimestampPrecisionConfig::Milliseconds => Duration::from_millis(stamp),
                    };

                    let timestamp = first_frame_start + duration_offset;

                    match &timestamp_config.datetime_format_string {
                        Some(fmt) => timestamp.format(fmt).to_string(),
                        None => timestamp.to_rfc3339(),
                    }
                }
                PrintStampConfig::UnixTimestamp(timestamp_config) => {
                    let epoch_offset = match timestamp_config.timestamp_precision {
                        PrintTimestampPrecisionConfig::Microseconds => Duration::from_micros(stamp),
                        PrintTimestampPrecisionConfig::Milliseconds => Duration::from_millis(stamp),
                    };

                    let timestamp = DateTime::UNIX_EPOCH + epoch_offset;

                    match &timestamp_config.datetime_format_string {
                        Some(fmt) => timestamp.format(fmt).to_string(),
                        None => timestamp.to_rfc3339(),
                    }
                }
            }
        }

        fn evaluate_statement(
            format_string: &ProcessedFormatString,
            decoded_values: &DecodedValues,
            append_newline: bool,
        ) -> Result<String, DeferredFormatError> {
            let deferred_values = convert_decoded_values(decoded_values)?;
            let mut expression_string = format_string.format_deferred(&deferred_values)?;

            if append_newline {
                expression_string.push('\n');
            }

            return Ok(expression_string);

            fn convert_decoded_values<'v>(
                decoded_values: &'v DecodedValues,
            ) -> Result<DeferredValues<'v>, DeferredFormatError> {
                let DecodedValues { positional, named } = decoded_values;

                let deferred_positional = convert_values(positional)?;
                let deferred_named = named
                    .iter()
                    .map(|(identifier, value)| Ok(((*identifier).clone(), convert_value(value)?)))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(DeferredValues::new(deferred_positional, deferred_named))
            }

            fn convert_values<'v>(decoded_values: &'v [Value]) -> Result<Vec<DeferredValue<'v>>, DeferredFormatError> {
                decoded_values.iter().map(convert_value).collect()
            }

            fn convert_value<'v>(decoded_value: &'v Value) -> Result<DeferredValue<'v>, DeferredFormatError> {
                let value = match decoded_value {
                    Value::Boolean(value) => DeferredValue::Boolean(*value),
                    Value::Usize(value) => DeferredValue::U64(*value),
                    Value::U8(value) => DeferredValue::U8(*value),
                    Value::U16(value) => DeferredValue::U16(*value),
                    Value::U32(value) => DeferredValue::U32(*value),
                    Value::U64(value) => DeferredValue::U64(*value),
                    Value::U128(value) => DeferredValue::U128(*value),
                    Value::Isize(value) => DeferredValue::I64(*value),
                    Value::I8(value) => DeferredValue::I8(*value),
                    Value::I16(value) => DeferredValue::I16(*value),
                    Value::I32(value) => DeferredValue::I32(*value),
                    Value::I64(value) => DeferredValue::I64(*value),
                    Value::I128(value) => DeferredValue::I128(*value),
                    Value::F32(value) => DeferredValue::F32(*value),
                    Value::F64(value) => DeferredValue::F64(*value),
                    Value::Char(value) => DeferredValue::Char(*value),
                    Value::String(value) => DeferredValue::String(value.into()),
                    Value::List(values) => convert_values(values).map(DeferredValue::List)?,
                    Value::Tuple(values) => convert_values(values).map(DeferredValue::Tuple)?,
                    Value::Type(value) => {
                        let TypeStructureValue { name, variant } = value;

                        let deferred_variant = match variant {
                            TypeStructureVariantValue::Struct(decoded_struct_variant) => {
                                convert_struct_variant(decoded_struct_variant).map(DeferredTypeVariant::Struct)?
                            }
                            TypeStructureVariantValue::Enum((variant_name, variant_value)) => {
                                let deferred_struct_value = convert_struct_variant(variant_value)?;
                                DeferredTypeVariant::Enum((variant_name, deferred_struct_value))
                            }
                        };

                        DeferredValue::Type(DeferredTypeValue { name, variant: deferred_variant })
                    }
                    Value::NestedFormatExpression { expression, append_newline, decoded_values } => {
                        let pretty_string =
                            PrettyPrinter::evaluate_statement(expression, decoded_values, *append_newline)?;
                        DeferredValue::String(pretty_string.into())
                    }
                };

                Ok(value)
            }

            fn convert_struct_variant<'v>(
                decoded_struct_variant: &'v StructVariantValue,
            ) -> Result<DeferredStructVariant<'v>, DeferredFormatError> {
                let deferred_variant = match decoded_struct_variant {
                    StructVariantValue::Unit => DeferredStructVariant::Unit,
                    StructVariantValue::Tuple(values) => convert_values(values).map(DeferredStructVariant::Tuple)?,
                    StructVariantValue::Named(fields) => fields
                        .iter()
                        .map(|(field_name, field_value)| Ok((*field_name, convert_value(field_value)?)))
                        .collect::<Result<_, _>>()
                        .map(DeferredStructVariant::Named)?,
                };

                Ok(deferred_variant)
            }
        }
    }
}

pub(crate) use config::*;
pub mod config {
    use redefmt_args::{
        format_string::{FormatString, FormatStringParseErrorKind},
        identifier::AnyIdentifier,
        processor::{DynamicProcessorConfig, FormatProcessor, FormatProcessorError, ProcessedFormatString},
    };

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
        pub fn new(
            stamp_config: PrintStampConfig,
            log_format_str: Option<&str>,
        ) -> Result<Self, PrettyPrinterConfigError> {
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

    pub enum PrintStampConfig {
        Counter,
        /// Offset from datetime of first frame
        OffsetTimestamp(PrintTimestampConfig),
        UnixTimestamp(PrintTimestampConfig),
    }

    pub struct PrintTimestampConfig {
        pub timestamp_precision: PrintTimestampPrecisionConfig,
        /// `strftime` inspired string passed to `chrono`'s [`DateTime::format`][fmt]
        ///
        /// Defaults to RFC 3339 / ISO 8601 if none is provided.
        ///
        /// [fmt]: https://docs.rs/chrono/0.4.41/chrono/struct.DateTime.html#method.format
        pub datetime_format_string: Option<String>,
    }

    pub enum PrintTimestampPrecisionConfig {
        Microseconds,
        Milliseconds,
    }
}
