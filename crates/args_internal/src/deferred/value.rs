use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt::{Binary, Debug, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex};

use strum::IntoDiscriminant;

use crate::*;

#[derive(Debug, Clone, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display), strum(serialize_all = "lowercase"))]
pub enum DeferredValue<'a> {
    Boolean(bool),
    Usize(usize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Isize(isize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
    Char(char),
    String(&'a str),
    // Reused for array, vec and slice
    List(Vec<DeferredValue<'a>>),
    Tuple(Vec<DeferredValue<'a>>),
    Type(DeferredTypeValue<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeferredTypeValue<'a> {
    pub name: &'a str,
    pub variant: DeferredTypeVariant<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeferredTypeVariant<'a> {
    Struct(DeferredStructVariant<'a>),
    Enum((&'a str, DeferredStructVariant<'a>)),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeferredStructVariant<'a> {
    Unit,
    Tuple(&'a [DeferredValue<'a>]),
    Named(&'a [(&'a str, DeferredValue<'a>)]),
}

enum ValueClass {
    Numeric,
    Structure,
    Misc,
}

impl<'a> DeferredValue<'a> {
    pub fn evaluate(
        &self,
        string_buffer: &mut String,
        format_options: &FormatOptions,
        provided_args: &DeferredProvidedArgs,
    ) -> Result<(), DeferredFormatError> {
        let format_trait = format_options.format_trait;
        let use_alternate_form = format_options.use_alternate_form;
        let use_zero_padding = format_options.use_zero_padding;

        let width = self.resolve_width(format_options, provided_args)?;
        let precision = self.resolve_precision(format_options, provided_args)?;

        let value_string = match self {
            DeferredValue::Boolean(value) => match format_trait {
                FormatTrait::Display | FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    match value {
                        true => "true",
                        false => "false",
                    }
                    .to_string()
                }
                FormatTrait::Pointer => pointer_string(value, width, format_options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
            DeferredValue::Char(value) => match format_trait {
                FormatTrait::Display => value.to_string(),
                FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    format!("{value:?}")
                }
                FormatTrait::Pointer => pointer_string(value, width, format_options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
            DeferredValue::String(value) => match format_trait {
                FormatTrait::Display => value.to_string(),
                FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    format!("{value:?}")
                }
                FormatTrait::Pointer => pointer_string(value, width, format_options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
            DeferredValue::Usize(value) => integer_string(value, width, precision, format_options),
            DeferredValue::U8(value) => integer_string(value, width, precision, format_options),
            DeferredValue::U16(value) => integer_string(value, width, precision, format_options),
            DeferredValue::U32(value) => integer_string(value, width, precision, format_options),
            DeferredValue::U64(value) => integer_string(value, width, precision, format_options),
            DeferredValue::U128(value) => integer_string(value, width, precision, format_options),
            DeferredValue::Isize(value) => integer_string(value, width, precision, format_options),
            DeferredValue::I8(value) => integer_string(value, width, precision, format_options),
            DeferredValue::I16(value) => integer_string(value, width, precision, format_options),
            DeferredValue::I32(value) => integer_string(value, width, precision, format_options),
            DeferredValue::I64(value) => integer_string(value, width, precision, format_options),
            DeferredValue::I128(value) => integer_string(value, width, precision, format_options),
            DeferredValue::F32(value) => float_string(self.discriminant(), value, width, precision, format_options)?,
            DeferredValue::F64(value) => float_string(self.discriminant(), value, width, precision, format_options)?,
            DeferredValue::List(values) => match format_trait {
                FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    match use_alternate_form {
                        true => todo!(),
                        false => todo!(),
                    }
                }
                FormatTrait::Pointer => pointer_string(values, width, format_options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
            DeferredValue::Tuple(values) => match format_trait {
                FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    match use_alternate_form {
                        true => todo!(),
                        false => todo!(),
                    }
                }
                FormatTrait::Pointer => pointer_string(values, width, format_options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
            DeferredValue::Type(type_value) => match format_trait {
                FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    match use_alternate_form {
                        true => todo!(),
                        false => todo!(),
                    }
                }
                FormatTrait::Pointer => pointer_string(type_value, width, format_options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
        };

        match self.handle_width(use_zero_padding) {
            true => {
                todo!()
            }
            false => {
                string_buffer.push_str(&value_string);
            }
        }

        Ok(())
    }

    fn handle_width(&self, use_zero_padding: bool) -> bool {
        match self.value_class() {
            ValueClass::Numeric => !use_zero_padding,
            ValueClass::Structure => false,
            ValueClass::Misc => true,
        }
    }

    fn value_class(&self) -> ValueClass {
        match self {
            DeferredValue::Boolean(_) | DeferredValue::Char(_) | DeferredValue::String(_) => ValueClass::Misc,
            DeferredValue::List(_) | DeferredValue::Tuple(_) | DeferredValue::Type(_) => ValueClass::Structure,
            DeferredValue::Usize(_)
            | DeferredValue::U8(_)
            | DeferredValue::U16(_)
            | DeferredValue::U32(_)
            | DeferredValue::U64(_)
            | DeferredValue::U128(_)
            | DeferredValue::Isize(_)
            | DeferredValue::I8(_)
            | DeferredValue::I16(_)
            | DeferredValue::I32(_)
            | DeferredValue::I64(_)
            | DeferredValue::I128(_)
            | DeferredValue::F32(_)
            | DeferredValue::F64(_) => ValueClass::Numeric,
        }
    }

    fn resolve_width(
        &self,
        format_options: &FormatOptions,
        provided_args: &DeferredProvidedArgs,
    ) -> Result<usize, DeferredFormatError> {
        let width = match &format_options.width {
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
        &self,
        format_options: &FormatOptions,
        provided_args: &DeferredProvidedArgs,
    ) -> Result<Option<usize>, DeferredFormatError> {
        let precision = match &format_options.precision {
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

// NOTE: Most of this could be removed if and
// when `formatting_options` is stabilized.
// https://github.com/rust-lang/rust/issues/118117

fn float_string<T: Copy + Display + Debug + LowerExp + UpperExp>(
    discriminant: DeferredValueDiscriminants,
    float: T,
    width: usize,
    precision: Option<usize>,
    format_options: &FormatOptions,
) -> Result<String, DeferredFormatError> {
    let FormatOptions { sign, use_zero_padding, format_trait, .. } = format_options;

    #[rustfmt::skip]
    let string = match (sign == &Some(Sign::Plus), use_zero_padding, precision, format_trait) {
        (false, false, None, FormatTrait::Display) => format!("{float}"),
        (false, true, None, FormatTrait::Display) => format!("{float:0width$}"),
        (true, false, None, FormatTrait::Display) => format!("{float:+}"),
        (true, true, None, FormatTrait::Display) => format!("{float:+0width$}"),
        (false, false, None, FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex) => { format!("{float:?}") }
        (false, true, None, FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex) => { format!("{float:0width$?}") }
        (true, false, None, FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex) => { format!("{float:+?}") }
        (true, true, None, FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex) => { format!("{float:+0width$?}") }
        (false, false, Some(precision), FormatTrait::Display) => format!("{float:.precision$}"),
        (false, true, Some(precision), FormatTrait::Display) => format!("{float:0width$.precision$}"),
        (true, false, Some(precision), FormatTrait::Display) => format!("{float:+.precision$}"),
        (true, true, Some(precision), FormatTrait::Display) => format!("{float:+0width$.precision$}"),
        (false, false, Some(precision), FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex) => { format!("{float:.precision$?}") }
        (false, true, Some(precision), FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex) => { format!("{float:0width$.precision$?}") }
        (true, false, Some(precision), FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex) => { format!("{float:+.precision$?}") }
        (true, true, Some(precision), FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex) => { format!("{float:+0width$.precision$?}") }
        (sign, zero_padding, precision, FormatTrait::UpperExp) => exp_string(float, true, sign, *zero_padding, width, precision),
        (sign, zero_padding, precision, FormatTrait::LowerExp) => exp_string(float, false, sign, *zero_padding, width, precision),
        (_, _,_, FormatTrait::Pointer) => pointer_string(float, width, format_options),
        _ => {
            return Err(DeferredFormatError::FormatNotImplemented(*format_trait, discriminant));
        }
    };

    Ok(string)
}

fn integer_string<T: Display + Binary + LowerHex + UpperHex + Octal + LowerExp + UpperExp>(
    numeric: T,
    width: usize,
    precision: Option<usize>,
    format_options: &FormatOptions,
) -> String {
    let FormatOptions { sign, use_alternate_form, use_zero_padding, format_trait, .. } = format_options;

    match (
        sign == &Some(Sign::Plus),
        use_alternate_form,
        use_zero_padding,
        format_trait,
    ) {
        (_, _, _, FormatTrait::Pointer) => pointer_string(numeric, width, format_options),
        (false, _, false, FormatTrait::Display | FormatTrait::Debug) => format!("{numeric}"),
        (false, _, true, FormatTrait::Display | FormatTrait::Debug) => format!("{numeric:0width$}"),
        (true, _, true, FormatTrait::Display | FormatTrait::Debug) => format!("{numeric:+}"),
        (true, _, false, FormatTrait::Display | FormatTrait::Debug) => format!("{numeric:+0width$}"),
        (false, false, false, FormatTrait::DebugUpperHex | FormatTrait::UpperHex) => format!("{numeric:X}"),
        (false, false, true, FormatTrait::DebugUpperHex | FormatTrait::UpperHex) => format!("{numeric:0width$X}"),
        (false, true, false, FormatTrait::DebugUpperHex | FormatTrait::UpperHex) => format!("{numeric:#X}"),
        (false, true, true, FormatTrait::DebugUpperHex | FormatTrait::UpperHex) => format!("{numeric:#0width$X}"),
        (true, false, false, FormatTrait::DebugUpperHex | FormatTrait::UpperHex) => format!("{numeric:+X}"),
        (true, false, true, FormatTrait::DebugUpperHex | FormatTrait::UpperHex) => format!("{numeric:+0width$X}"),
        (true, true, false, FormatTrait::DebugUpperHex | FormatTrait::UpperHex) => format!("{numeric:+#X}"),
        (true, true, true, FormatTrait::DebugUpperHex | FormatTrait::UpperHex) => format!("{numeric:+#0width$X}"),
        (false, false, false, FormatTrait::DebugLowerHex | FormatTrait::LowerHex) => format!("{numeric:x}"),
        (false, false, true, FormatTrait::DebugLowerHex | FormatTrait::LowerHex) => format!("{numeric:0width$x}"),
        (false, true, false, FormatTrait::DebugLowerHex | FormatTrait::LowerHex) => format!("{numeric:#x}"),
        (false, true, true, FormatTrait::DebugLowerHex | FormatTrait::LowerHex) => format!("{numeric:#0width$x}"),
        (true, false, false, FormatTrait::DebugLowerHex | FormatTrait::LowerHex) => format!("{numeric:+x}"),
        (true, false, true, FormatTrait::DebugLowerHex | FormatTrait::LowerHex) => format!("{numeric:+0width$x}"),
        (true, true, false, FormatTrait::DebugLowerHex | FormatTrait::LowerHex) => format!("{numeric:+#x}"),
        (true, true, true, FormatTrait::DebugLowerHex | FormatTrait::LowerHex) => format!("{numeric:+#0width$x}"),
        (false, false, false, FormatTrait::Octal) => format!("{numeric:o}"),
        (false, false, true, FormatTrait::Octal) => format!("{numeric:0width$o}"),
        (false, true, false, FormatTrait::Octal) => format!("{numeric:#o}"),
        (false, true, true, FormatTrait::Octal) => format!("{numeric:#0width$o}"),
        (true, false, false, FormatTrait::Octal) => format!("{numeric:+o}"),
        (true, false, true, FormatTrait::Octal) => format!("{numeric:+0width$o}"),
        (true, true, false, FormatTrait::Octal) => format!("{numeric:+#o}"),
        (true, true, true, FormatTrait::Octal) => format!("{numeric:+#0width$o}"),
        (false, false, false, FormatTrait::Binary) => format!("{numeric:b}"),
        (false, false, true, FormatTrait::Binary) => format!("{numeric:0width$b}"),
        (false, true, false, FormatTrait::Binary) => format!("{numeric:#b}"),
        (false, true, true, FormatTrait::Binary) => format!("{numeric:#0width$b}"),
        (true, false, false, FormatTrait::Binary) => format!("{numeric:+b}"),
        (true, false, true, FormatTrait::Binary) => format!("{numeric:+0width$b}"),
        (true, true, false, FormatTrait::Binary) => format!("{numeric:+#b}"),
        (true, true, true, FormatTrait::Binary) => format!("{numeric:+#0width$b}"),
        (sign, _, zero_padding, FormatTrait::LowerExp) => {
            exp_string(numeric, false, sign, *zero_padding, width, precision)
        }
        (sign, _, zero_padding, FormatTrait::UpperExp) => {
            exp_string(numeric, true, sign, *zero_padding, width, precision)
        }
    }
}

fn pointer_string<T>(t: T, width: usize, format_options: &FormatOptions) -> String {
    let FormatOptions { sign, use_alternate_form, use_zero_padding, .. } = format_options;

    let pointer: *const T = &t;

    match (sign == &Some(Sign::Plus), use_alternate_form, use_zero_padding) {
        (true, true, true) => format!("{pointer:+#0width$p}"),
        (true, true, false) => format!("{pointer:+#p}"),
        (true, false, true) => format!("{pointer:+0width$p}"),
        (true, false, false) => format!("{pointer:+p}"),
        (false, true, true) => format!("{pointer:#0width$p}"),
        (false, true, false) => format!("{pointer:#p}"),
        (false, false, true) => format!("{pointer:0width$p}"),
        (false, false, false) => format!("{pointer:p}"),
    }
}

fn exp_string<T: UpperExp + LowerExp>(
    t: T,
    upper: bool,
    sign: bool,
    zero_padding: bool,
    width: usize,
    precision: Option<usize>,
) -> String {
    match (upper, sign, zero_padding, precision) {
        (false, false, false, None) => format!("{t:e}"),
        (false, false, true, None) => format!("{t:0width$e}"),
        (false, true, false, None) => format!("{t:+e}"),
        (false, true, true, None) => format!("{t:+0width$e}"),
        (true, false, false, None) => format!("{t:E}"),
        (true, false, true, None) => format!("{t:0width$E}"),
        (true, true, false, None) => format!("{t:+E}"),
        (true, true, true, None) => format!("{t:+0width$E}"),
        (false, false, false, Some(precision)) => format!("{t:.precision$e}"),
        (false, false, true, Some(precision)) => format!("{t:0width$.precision$e}"),
        (false, true, false, Some(precision)) => format!("{t:+.precision$e}"),
        (false, true, true, Some(precision)) => format!("{t:+0width$.precision$e}"),
        (true, false, false, Some(precision)) => format!("{t:.precision$E}"),
        (true, false, true, Some(precision)) => format!("{t:0width$.precision$E}"),
        (true, true, false, Some(precision)) => format!("{t:+.precision$E}"),
        (true, true, true, Some(precision)) => format!("{t:+0width$.precision$E}"),
    }
}
