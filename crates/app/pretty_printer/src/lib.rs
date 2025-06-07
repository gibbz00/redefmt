//! # `redefmt-pretty-printer`

// TEMP:
#![allow(missing_docs)]

pub use printer::PrettyPrinter;
mod printer {
    use std::time::Duration;

    use chrono::{DateTime, Utc};
    use redefmt_args::{
        deferred::{
            DeferredFormatConfig, DeferredFormatError, DeferredStructVariant, DeferredTypeValue, DeferredTypeVariant,
            DeferredValue, DeferredValues,
        },
        identifier::AnyIdentifier,
        processor::ProcessedFormatString,
    };
    use redefmt_decoder::{
        RedefmtFrame,
        values::{
            DecodedValues, StructVariantValue, TypeStructureValue, TypeStructureVariantValue, Value,
            WriteStatementValue,
        },
    };

    use crate::*;

    const FORMAT_DEFERRED_CONFIG: DeferredFormatConfig = DeferredFormatConfig {
        allow_non_usize_precision_value: true,
        allow_non_usize_width_value: true,
    };

    pub struct PrettyPrinter {
        first_frame_start: Option<DateTime<Utc>>,
        config: PrettyPrinterConfig,
    }

    impl PrettyPrinter {
        pub fn new(config: PrettyPrinterConfig) -> Self {
            Self { first_frame_start: None, config }
        }

        pub fn format(&mut self, redefmt_frame: RedefmtFrame) -> Result<String, DeferredFormatError> {
            let RedefmtFrame {
                level,
                stamp,
                crate_name,
                file_name,
                file_line,
                format_string,
                append_newline,
                decoded_values,
            } = redefmt_frame;

            let stamp = stamp.map(|stamp| self.evaluate_stamp(stamp)).unwrap_or_default();

            let statement = Self::evaluate_statement(format_string, &decoded_values, append_newline)?;

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

            self.config
                .log_format_string
                .format_deferred(&deferred_values, &FORMAT_DEFERRED_CONFIG)
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
            let mut expression_string = format_string.format_deferred(&deferred_values, &FORMAT_DEFERRED_CONFIG)?;

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
                    Value::WriteStatements(write_statements) => {
                        let mut write_statements_string = String::new();

                        for WriteStatementValue { expression, append_newline, decoded_values } in write_statements {
                            let pretty_string =
                                PrettyPrinter::evaluate_statement(expression, decoded_values, *append_newline)?;
                            write_statements_string.push_str(&pretty_string);
                        }

                        DeferredValue::String(write_statements_string.into())
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
        /// Construct a new pretty printer configuration using the default log statement format
        ///
        /// Defaults format: `"{stamp} [{level}] - {crate}: {statement}"`
        ///
        /// Use [`Self::new_with_format`] for custom log statement formats.
        pub fn new(stamp_config: PrintStampConfig) -> Self {
            Self::new_with_format(stamp_config, DEFAULT_LOG_FORMAT_STRING).expect("invalid default log format string")
        }

        /// Construct a new pretty printer configuration with a custom log statement format
        ///
        /// Format string can use the following  named parameters: "stamp",
        /// "level", "crate", "file", "line", "statement". Order can be relied
        /// on if the format string uses positional arguments.
        pub fn new_with_format(
            stamp_config: PrintStampConfig,
            log_format_str: &str,
        ) -> Result<Self, PrettyPrinterConfigError> {
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

#[cfg(test)]
mod tests {
    use redefmt::{Level, logger::GlobalLogger};
    use redefmt_core::{SharedTestDispatcher, logger::TestStamper};
    use redefmt_decoder::{RedefmtDecoder, RedefmtDecoderCache};

    use crate::*;

    macro_rules! assert_print {
        ($dispatcher:expr, $decoder:expr, $printer:expr, $log_expr:expr, $expected:literal) => {{
            $log_expr;
            let expected = format!($expected);
            assert_print_impl(&mut $dispatcher, &mut $decoder, &mut $printer, &expected);
        }};
    }

    fn assert_print_impl(
        dispatcher: &mut SharedTestDispatcher,
        decoder: &mut RedefmtDecoder,
        printer: &mut PrettyPrinter,
        expected: &str,
    ) {
        let frame = decoder.decode(&mut dispatcher.take_bytes()).unwrap().unwrap();
        let actual = printer.format(frame).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    #[rustfmt::skip]
    fn end_to_end() {
        static STAMPER: TestStamper = TestStamper::new();
        let dispatcher = SharedTestDispatcher::new();
        GlobalLogger::init_alloc_logger(dispatcher.clone(), Some(&STAMPER)).unwrap();

        let decoder_cache = RedefmtDecoderCache::default();
        let decoder = RedefmtDecoder::new(&decoder_cache).unwrap();

        let printer_config = PrettyPrinterConfig::new(PrintStampConfig::Counter);
        let printer = PrettyPrinter::new(printer_config);

        let value = 10;
        let crate_name = env!("CARGO_PKG_NAME");

        // Assertions in same test as global logger may only be set once.

        let mut l = dispatcher;
        let mut d = decoder;
        let mut p = printer;

        assert_print!(l, d, p, redefmt::print!("{value}"), "0 [NONE] - {crate_name}: {value}");
        assert_print!(l, d, p, redefmt::println!("{value}"), "1 [NONE] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::trace!("{value}"), "2 [TRACE] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::debug!("{value}"), "3 [DEBUG] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::info!("{value}"), "4 [INFO] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::warn!("{value}"), "5 [WARN] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::error!("{value}"), "6 [ERROR] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::log!(Level::Trace, "{value}"), "7 [TRACE] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::log!(Level::Debug, "{value}"), "8 [DEBUG] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::log!(Level::Info, "{value}"), "9 [INFO] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::log!(Level::Warn, "{value}"), "10 [WARN] - {crate_name}: {value}\n");
        assert_print!(l, d, p, redefmt::log!(Level::Error, "{value}"), "11 [ERROR] - {crate_name}: {value}\n");

        // level can be full-blown expression
        assert_print!(
            l, d, p,
            redefmt::log!(if true { redefmt::Level::Info } else { redefmt::Level::Error }, "{value}"),
            "12 [INFO] - {crate_name}: {value}\n"
        );

        // manual impl
        struct FooWrite;
        impl redefmt::Format for FooWrite {
            fn fmt(&self, f: &mut redefmt::Formatter) -> ::core::fmt::Result {
                let mut s = f.write_statements();
                redefmt::writeln!(s, "x")?;
                redefmt::write!(s, "y")?;
                Ok(())
            }
        }

        // note how FooWrite can be captured and created at the same time
        assert_print!(l, d, p, redefmt::print!("{FooWrite:?}"), "13 [NONE] - {crate_name}: \"x\\ny\"");

        // derive unit struct
        #[derive(redefmt::Format)]
        struct FooUnit;
        let value = FooUnit;
        assert_print!(l, d, p, redefmt::print!("{value:?}"), "14 [NONE] - {crate_name}: FooUnit");

        // generics
        #[derive(redefmt::Format)]
        struct FooGeneric<T>(T);
        assert_print!(l, d, p, redefmt::print!("{:?}", FooGeneric(1)), "15 [NONE] - {crate_name}: FooGeneric(1)");
        assert_print!(l, d, p, redefmt::print!("{:?}", FooGeneric("x")), "16 [NONE] - {crate_name}: FooGeneric(\"x\")");


        // derive named struct without fields
        #[derive(redefmt::Format)]
        struct FooNamed {}
        let value = FooNamed {};
        assert_print!(l, d, p, redefmt::print!("{value:?}"), "17 [NONE] - {crate_name}: FooNamed");

        // derive named struct with fields
        #[derive(redefmt::Format)]
        struct BarNamed {
            a: FooUnit,
            b: FooUnit,
        }
        let value = BarNamed { a: FooUnit, b: FooUnit};
        assert_print!(l, d, p, redefmt::print!("{value:?}"), "18 [NONE] - {crate_name}: BarNamed {{ a: FooUnit, b: FooUnit }}");

        // derive empty tuple struct
        #[derive(redefmt::Format)]
        struct FooTuple();
        let value = FooTuple();
        assert_print!(l, d, p, redefmt::print!("{value:?}"), "19 [NONE] - {crate_name}: FooTuple");

        // derive non-empty tuple struct
        #[derive(redefmt::Format)]
        struct BarTuple(FooUnit, FooUnit);
        let value = BarTuple(FooUnit, FooUnit);
        assert_print!(l, d, p, redefmt::print!("{value:?}"), "20 [NONE] - {crate_name}: BarTuple(FooUnit, FooUnit)");

        // derive on enum
        #[derive(redefmt::Format)]
        enum FooEnum {
            Unit,
            Tuple(usize, usize),
            Named { a: usize, b: usize },
        }
        let value = FooEnum::Unit;
        assert_print!(l, d, p, redefmt::print!("{value:?}"), "21 [NONE] - {crate_name}: Unit");
        let value = FooEnum::Tuple(1, 2);
        assert_print!(l, d, p, redefmt::print!("{value:?}"), "22 [NONE] - {crate_name}: Tuple(1, 2)");
        let value = FooEnum::Named { a: 1, b: 2 };
        assert_print!(l, d, p, redefmt::print!("{value:?}"), "23 [NONE] - {crate_name}: Named {{ a: 1, b: 2 }}");

        // derive on enum with no variants
        #[allow(unused)]
        #[derive(redefmt::Format)]
        enum FooEnumInfallible {}
    }
}
