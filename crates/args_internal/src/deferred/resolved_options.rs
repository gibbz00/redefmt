use strum::IntoDiscriminant;

use crate::*;

pub struct ResolvedFormatOptions {
    pub(super) align: Option<FormatAlign>,
    pub(super) sign: bool,
    pub(super) use_alternate_form: bool,
    pub(super) use_zero_padding: bool,
    pub(super) width: usize,
    pub(super) precision: Option<usize>,
    pub(super) format_trait: FormatTrait,
}

impl ResolvedFormatOptions {
    pub fn resolve(
        format_options: &FormatOptions,
        provided_args: &DeferredProvidedArgs,
    ) -> Result<Self, DeferredFormatError> {
        let align = format_options.align;
        let sign = format_options.sign == Some(Sign::Plus);
        let use_alternate_form = format_options.use_alternate_form;
        let use_zero_padding = format_options.use_zero_padding;
        let width = Self::resolve_width(format_options.width.as_ref(), provided_args)?;
        let precision = Self::resolve_precision(format_options.precision.as_ref(), provided_args)?;
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
        provided_args: &DeferredProvidedArgs,
    ) -> Result<usize, DeferredFormatError> {
        let width = match width_option {
            Some(width_arg) => match width_arg {
                FormatCount::Integer(int) => *int,
                FormatCount::Argument(format_argument) => match provided_args.get(format_argument)? {
                    DeferredValue::Usize(arg_value) => *arg_value,
                    other => {
                        return Err(DeferredFormatError::InvalidArgType(
                            DeferredValueDiscriminants::Usize,
                            other.discriminant(),
                        ));
                    }
                },
            },
            None => 0,
        };

        Ok(width)
    }

    fn resolve_precision(
        precision_option: Option<&FormatPrecision>,
        provided_args: &DeferredProvidedArgs,
    ) -> Result<Option<usize>, DeferredFormatError> {
        let precision = match precision_option {
            Some(precision) => match precision {
                FormatPrecision::Count(format_count) => match format_count {
                    FormatCount::Integer(integer) => Some(*integer),
                    FormatCount::Argument(format_argument) => match provided_args.get(format_argument)? {
                        DeferredValue::Usize(arg_value) => Some(*arg_value),
                        other => {
                            return Err(DeferredFormatError::InvalidArgType(
                                DeferredValueDiscriminants::Usize,
                                other.discriminant(),
                            ));
                        }
                    },
                },
                FormatPrecision::NextArgument => unreachable!("should have been disambiguated by argument resolver"),
            },
            None => None,
        };

        Ok(precision)
    }
}
