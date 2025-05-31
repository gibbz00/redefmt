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

#[derive(PartialEq)]
enum ValueClass {
    Numeric,
    Structure,
    Misc,
}

#[derive(Default)]
struct EvaluationContext {
    indentation: usize,
}

impl<'a> DeferredValue<'a> {
    pub fn evaluate(
        &self,
        string_buffer: &mut String,
        format_options: &FormatOptions,
        provided_args: &DeferredProvidedArgs,
    ) -> Result<(), DeferredFormatError> {
        let mut evaltation_context = EvaluationContext::default();
        let options = ResolvedFormatOptions::resolve(format_options, provided_args)?;
        self.evaluate_impl(string_buffer, &mut evaltation_context, &options)
    }

    fn evaluate_impl(
        &self,
        string_buffer: &mut String,
        evaluation_context: &mut EvaluationContext,
        options: &ResolvedFormatOptions,
    ) -> Result<(), DeferredFormatError> {
        let format_trait = options.format_trait;

        let value_string = match self {
            DeferredValue::Boolean(value) => match options.format_trait {
                FormatTrait::Display | FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    match value {
                        true => "true",
                        false => "false",
                    }
                    .to_string()
                }
                FormatTrait::Pointer => pointer_string(value, options),
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
                FormatTrait::Pointer => pointer_string(value, options),
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
                FormatTrait::Pointer => pointer_string(value, options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
            DeferredValue::Usize(value) => integer_string(value, options),
            DeferredValue::U8(value) => integer_string(value, options),
            DeferredValue::U16(value) => integer_string(value, options),
            DeferredValue::U32(value) => integer_string(value, options),
            DeferredValue::U64(value) => integer_string(value, options),
            DeferredValue::U128(value) => integer_string(value, options),
            DeferredValue::Isize(value) => integer_string(value, options),
            DeferredValue::I8(value) => integer_string(value, options),
            DeferredValue::I16(value) => integer_string(value, options),
            DeferredValue::I32(value) => integer_string(value, options),
            DeferredValue::I64(value) => integer_string(value, options),
            DeferredValue::I128(value) => integer_string(value, options),
            DeferredValue::F32(value) => float_string(self.discriminant(), value, options)?,
            DeferredValue::F64(value) => float_string(self.discriminant(), value, options)?,
            DeferredValue::List(values) => match format_trait {
                FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    match options.use_alternate_form {
                        true => todo!(),
                        false => todo!(),
                    }
                }
                FormatTrait::Pointer => pointer_string(values, options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
            DeferredValue::Tuple(values) => match format_trait {
                FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    return tuple_string(string_buffer, values, evaluation_context, options);
                }
                FormatTrait::Pointer => pointer_string(values, options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
            DeferredValue::Type(type_value) => match format_trait {
                FormatTrait::Debug | FormatTrait::DebugLowerHex | FormatTrait::DebugUpperHex => {
                    match options.use_alternate_form {
                        true => todo!(),
                        false => todo!(),
                    }
                }
                FormatTrait::Pointer => pointer_string(type_value, options),
                _ => {
                    return Err(DeferredFormatError::FormatNotImplemented(
                        format_trait,
                        self.discriminant(),
                    ));
                }
            },
        };

        let value_string = pipeline_length(self.value_class(), value_string, options);

        string_buffer.push_str(&value_string);

        Ok(())
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
}

// NOTE: Most of this could be removed if and
// when `formatting_options` is stabilized.
// https://github.com/rust-lang/rust/issues/118117

fn float_string<T: Copy + Display + Debug + LowerExp + UpperExp>(
    discriminant: DeferredValueDiscriminants,
    float: T,
    options: &ResolvedFormatOptions,
) -> Result<String, DeferredFormatError> {
    let ResolvedFormatOptions { sign, use_zero_padding, format_trait, width, precision, .. } = options;

    #[rustfmt::skip]
    let string = match (sign , use_zero_padding, precision, format_trait) {
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
        (_, _, _, FormatTrait::UpperExp) => exp_string(float, true, options),
        (_, _, _, FormatTrait::LowerExp) => exp_string(float, false, options),
        (_, _,_, FormatTrait::Pointer) => pointer_string(float, options),
        _ => {
            return Err(DeferredFormatError::FormatNotImplemented(*format_trait, discriminant));
        }
    };

    Ok(string)
}

fn integer_string<T: Display + Binary + LowerHex + UpperHex + Octal + LowerExp + UpperExp>(
    numeric: T,
    options: &ResolvedFormatOptions,
) -> String {
    let ResolvedFormatOptions { sign, use_alternate_form, use_zero_padding, format_trait, width, .. } = options;

    match (sign, use_alternate_form, use_zero_padding, format_trait) {
        (_, _, _, FormatTrait::Pointer) => pointer_string(numeric, options),
        (false, _, false, FormatTrait::Display | FormatTrait::Debug) => format!("{numeric}"),
        (false, _, true, FormatTrait::Display | FormatTrait::Debug) => format!("{numeric:0width$}"),
        (true, _, false, FormatTrait::Display | FormatTrait::Debug) => format!("{numeric:+}"),
        (true, _, true, FormatTrait::Display | FormatTrait::Debug) => format!("{numeric:+0width$}"),
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
        (_, _, _, FormatTrait::LowerExp) => exp_string(numeric, false, options),
        (_, _, _, FormatTrait::UpperExp) => exp_string(numeric, true, options),
    }
}

fn pointer_string<T>(t: T, options: &ResolvedFormatOptions) -> String {
    let ResolvedFormatOptions { sign, use_alternate_form, use_zero_padding, width, .. } = options;

    let pointer: *const T = &t;

    match (sign, use_alternate_form, use_zero_padding) {
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

fn exp_string<T: UpperExp + LowerExp>(t: T, upper: bool, options: &ResolvedFormatOptions) -> String {
    let ResolvedFormatOptions { sign, use_zero_padding, width, precision, .. } = options;

    match (upper, sign, use_zero_padding, precision) {
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

fn tuple_string(
    string_buffer: &mut String,
    elements: &[DeferredValue],
    evaluation_context: &mut EvaluationContext,
    options: &ResolvedFormatOptions,
) -> Result<(), DeferredFormatError> {
    let pretty = options.use_alternate_form;

    string_buffer.push('(');

    if pretty {
        string_buffer.push('\n');
        evaluation_context.indentation += 1;
    }

    for (index, element) in elements.iter().enumerate() {
        if pretty {
            (0..evaluation_context.indentation).for_each(|_| string_buffer.push('\t'));
        }

        element.evaluate_impl(string_buffer, evaluation_context, options)?;

        if pretty {
            string_buffer.push(',');
            string_buffer.push('\n');
        } else if index + 1 != elements.len() {
            string_buffer.push_str(", ");
        }
    }

    if pretty {
        evaluation_context.indentation -= 1;
        (0..evaluation_context.indentation).for_each(|_| string_buffer.push('\t'));
    }

    string_buffer.push(')');

    Ok(())
}

fn pipeline_length(value_class: ValueClass, mut value_string: String, options: &ResolvedFormatOptions) -> String {
    let ResolvedFormatOptions { align, use_zero_padding, width, precision, .. } = options;

    let mut chars_count = value_string.chars().count();

    if let Some(precision) = precision
        && chars_count > *precision
        && value_class == ValueClass::Misc
    {
        let chars_to_pop = chars_count - precision;

        (0..chars_to_pop).for_each(|_| {
            value_string.pop();
        });

        chars_count -= chars_to_pop;
    }

    let apply_width = match value_class {
        ValueClass::Numeric => !use_zero_padding,
        ValueClass::Structure => false,
        ValueClass::Misc => true,
    };

    match apply_width && chars_count < *width {
        false => value_string,
        true => {
            let align = align.unwrap_or_else(|| FormatAlign {
                alignment: match value_class == ValueClass::Numeric {
                    true => Alignment::Right,
                    false => Alignment::Left,
                },
                character: None,
            });

            let pad_char = align.character.unwrap_or(' ');
            let pad_count = width - chars_count;

            match align.alignment {
                Alignment::Left => {
                    (0..pad_count).for_each(|_| value_string.push(pad_char));
                    value_string
                }
                Alignment::Center => {
                    let left_pad_count = pad_count / 2;
                    let right_pad_count = pad_count - left_pad_count;

                    let mut padded_string = core::iter::repeat_n(pad_char, left_pad_count).collect::<String>();
                    padded_string.push_str(&value_string);
                    (0..right_pad_count).for_each(|_| padded_string.push(pad_char));

                    padded_string
                }
                Alignment::Right => {
                    let mut padded_string = core::iter::repeat_n(pad_char, pad_count).collect::<String>();
                    padded_string.push_str(&value_string);
                    padded_string
                }
            }
        }
    }
}
