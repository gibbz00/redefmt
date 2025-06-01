//! # `redefmt-printer`

// TEMP:
#![allow(missing_docs)]

pub mod configuration;
pub(crate) use configuration::*;

mod pretty {
    use std::io::Write;

    use redefmt_args::deferred::{
        DeferredFormatExpression, DeferredProvidedArgs, DeferredStructVariant, DeferredTypeValue, DeferredTypeVariant,
        DeferredValue,
    };
    use redefmt_decoder::{
        RedefmtFrame,
        values::{DecodedValues, StructVariantValue, TypeStructureValue, TypeStructureVariantValue, Value},
    };

    use crate::*;

    #[derive(Debug, thiserror::Error)]
    pub enum RedefmtPrinterError {
        #[error("encountered I/O error when writing to output")]
        Io(std::io::Error),
    }

    pub fn pretty_print_frame(
        output: &mut impl Write,
        redefmt_frame: RedefmtFrame,
        config: &PrinterConfig,
    ) -> Result<(), RedefmtPrinterError> {
        let RedefmtFrame {
            level,
            stamp,
            file_name,
            file_line,
            format_expression,
            append_newline,
            decoded_values,
        } = redefmt_frame;

        todo!()
    }

    fn evaluate_expression_recursive(
        expression: &DeferredFormatExpression,
        decoded_values: &DecodedValues,
        append_newline: bool,
    ) -> Result<String, RedefmtPrinterError> {
        let deferred_values = convert_provided(decoded_values)?;
        // expression.evaluate(provided_args);

        todo!()
    }

    fn convert_value<'v>(decoded_value: &'v Value) -> Result<DeferredValue<'v>, RedefmtPrinterError> {
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
            // TODO: recursive decent expand all nested expressions
            Value::NestedFormatExpression { expression, append_newline, decoded_values } => {
                let pretty_string = evaluate_expression_recursive(expression, decoded_values, *append_newline)?;
                DeferredValue::String(pretty_string.into())
            }
        };

        Ok(value)
    }

    fn convert_provided<'v>(
        decoded_values: &'v DecodedValues,
    ) -> Result<DeferredProvidedArgs<'v>, RedefmtPrinterError> {
        let DecodedValues { positional, named } = decoded_values;

        let deferred_positional = convert_values(&positional)?;

        let mut deferred_named = Vec::with_capacity(named.len());
        for (named_identifier, named_value) in named {
            let deferred_value = convert_value(&named_value)?;
            // IMPROVEMENT: remove clone?
            let name = (*named_identifier).clone();
            deferred_named.push((name, deferred_value));
        }

        Ok(DeferredProvidedArgs::new(deferred_positional, deferred_named))
    }

    fn convert_values<'v>(decoded_values: &'v [Value]) -> Result<Vec<DeferredValue<'v>>, RedefmtPrinterError> {
        let mut elements = Vec::new();

        for decoded_value in decoded_values {
            let deferred_value = convert_value(decoded_value)?;
            elements.push(deferred_value);
        }

        Ok(elements)
    }

    fn convert_struct_variant<'v>(
        decoded_struct_variant: &'v StructVariantValue,
    ) -> Result<DeferredStructVariant<'v>, RedefmtPrinterError> {
        let deferred_variant = match decoded_struct_variant {
            StructVariantValue::Unit => DeferredStructVariant::Unit,
            StructVariantValue::Tuple(values) => convert_values(values).map(DeferredStructVariant::Tuple)?,
            StructVariantValue::Named(fields) => {
                let mut deferred_fields = Vec::new();

                for (field_name, field_value) in fields {
                    let deferred_value = convert_value(field_value)?;
                    deferred_fields.push((*field_name, deferred_value));
                }

                DeferredStructVariant::Named(deferred_fields)
            }
        };

        Ok(deferred_variant)
    }
}

#[cfg(feature = "async-example")]
pub mod async_example {
    use std::io::Write;

    use anyhow::Context;
    use futures_util::TryStreamExt;
    use redefmt_decoder::{RedefmtDecoder, RedefmtDecoderCache};
    use tokio::io::AsyncRead;
    use tokio_util::codec::FramedRead;

    use crate::*;

    pub async fn run(input: impl AsyncRead, mut output: impl Write, config: PrinterConfig) -> anyhow::Result<()> {
        let decoder_cache = RedefmtDecoderCache::default();
        let decoder = RedefmtDecoder::new(&decoder_cache).context("failed to init decoder")?;

        let framed_read = FramedRead::new(input, decoder);

        pin_utils::pin_mut!(framed_read);

        while let Some(redefmt_frame) = framed_read.try_next().await.context("failed to decode redefmt frame")? {
            pretty::pretty_print_frame(&mut output, redefmt_frame, &config).context("failed to print frame")?;
        }

        Ok(())
    }
}
