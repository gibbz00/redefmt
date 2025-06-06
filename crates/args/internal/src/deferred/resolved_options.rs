use crate::*;

pub(crate) struct ResolvedFormatOptions {
    pub(super) align: Option<FormatAlign>,
    pub(super) sign: bool,
    pub(super) use_alternate_form: bool,
    pub(super) use_zero_padding: bool,
    pub(super) width: usize,
    pub(super) precision: Option<usize>,
    pub(super) format_trait: FormatTrait,
}

impl ResolvedFormatOptions {
    pub(crate) fn new(
        format_options: &FormatOptions,
        deferred_values: &DeferredValues,
        config: &DeferredFormatConfig,
    ) -> Result<Self, DeferredFormatError> {
        let align = format_options.align;
        let sign = format_options.sign == Some(Sign::Plus);
        let use_alternate_form = format_options.use_alternate_form;
        let use_zero_padding = format_options.use_zero_padding;

        let width = Self::resolve_width(
            format_options.width.as_ref(),
            deferred_values,
            config.allow_non_usize_width_value,
        )?;

        let precision = Self::resolve_precision(
            format_options.precision.as_ref(),
            deferred_values,
            config.allow_non_usize_precision_value,
        )?;

        let format_trait = format_options.format_trait;

        Ok(Self {
            align,
            sign,
            use_alternate_form,
            use_zero_padding,
            width,
            precision,
            format_trait,
        })
    }

    fn resolve_width(
        width_option: Option<&FormatCount>,
        deferred_values: &DeferredValues,
        allow_non_usize_width: bool,
    ) -> Result<usize, DeferredFormatError> {
        let width = match width_option {
            Some(width_arg) => match width_arg {
                FormatCount::Integer(int) => *int,
                FormatCount::Argument(format_argument) => deferred_values
                    .get(format_argument)
                    .and_then(|value| Self::get_usize(value, allow_non_usize_width))?,
            },
            None => 0,
        };

        Ok(width)
    }

    fn resolve_precision(
        precision_option: Option<&FormatPrecision>,
        deferred_values: &DeferredValues,
        allow_non_usize_precision: bool,
    ) -> Result<Option<usize>, DeferredFormatError> {
        let precision = match precision_option {
            Some(precision) => match precision {
                FormatPrecision::Count(format_count) => match format_count {
                    FormatCount::Integer(integer) => Some(*integer),
                    FormatCount::Argument(format_argument) => deferred_values
                        .get(format_argument)
                        .and_then(|value| Self::get_usize(value, allow_non_usize_precision))
                        .map(Some)?,
                },
                FormatPrecision::NextArgument => unreachable!("should have been disambiguated by argument resolver"),
            },
            None => None,
        };

        Ok(precision)
    }

    fn get_usize(deferred_value: &DeferredValue, allow_non_usize: bool) -> Result<usize, DeferredFormatError> {
        return match deferred_value {
            DeferredValue::Usize(value) => Ok(*value),
            DeferredValue::U8(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::U16(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::U32(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::U64(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::U128(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::Isize(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::I8(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::I16(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::I32(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::I64(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            DeferredValue::I128(value) if allow_non_usize => convert_int(*value, deferred_value.discriminant()),
            other => {
                return Err(DeferredFormatError::InvalidArgType(
                    DeferredValueDiscriminant::Usize,
                    other.discriminant(),
                ));
            }
        };

        fn convert_int<T: TryInto<usize>>(
            integer: T,
            discriminant: DeferredValueDiscriminant,
        ) -> Result<usize, DeferredFormatError> {
            integer
                .try_into()
                .map_err(|_| DeferredFormatError::UsizeConversion(discriminant))
        }
    }
}
