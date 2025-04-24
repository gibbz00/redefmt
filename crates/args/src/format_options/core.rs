use core::{iter::Peekable, str::CharIndices};

use crate::*;

type StrIter<'a> = Peekable<CharIndices<'a>>;

#[derive(Debug, Default, PartialEq, derive_getters::Getters)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct FormatOptions<'a> {
    align: Option<FormatAlign>,
    sign: Option<Sign>,
    #[cfg_attr(feature = "builder", builder(default))]
    use_alternate_form: bool,
    /// https://doc.rust-lang.org/std/fmt/index.html#sign0
    #[cfg_attr(feature = "builder", builder(default))]
    use_zero_padding: bool,
    width: Option<FormatCount<'a>>,
    precision: Option<FormatPrecision<'a>>,
    #[cfg_attr(feature = "builder", builder(default))]
    format_trait: FormatTrait,
}

impl<'a> FormatOptions<'a> {
    /// Context from `FormatSegment::parse`:
    /// - Does not include semicolon.
    /// - `str` may be empty.
    /// - Contains no trailing whitespace.
    pub(crate) fn parse(offset: usize, str: &'a str) -> Result<Self, ParseError> {
        let mut format_options = FormatOptions::default();
        let mut str_iter = str.char_indices().peekable();

        parse_from_align(offset, str, &mut str_iter, &mut format_options)?;

        Ok(format_options)
    }
}

fn parse_from_align<'a>(
    offset: usize,
    initial_str: &'a str,
    str_iter: &mut StrIter<'a>,
    format_options: &mut FormatOptions<'a>,
) -> Result<(), ParseError> {
    let Some((ch_index, ch)) = str_iter.next() else {
        return Ok(());
    };

    let alignment = match ch {
        '<' => Some(Alignment::Left),
        '^' => Some(Alignment::Center),
        '>' => Some(Alignment::Right),
        _ => None,
    };

    match alignment {
        Some(alignment) => {
            format_options.align = Some(FormatAlign::new(alignment, None));
        }
        None => {
            if let Some(alignment) = str_iter.peek().and_then(|(_, ch)| Alignment::from_char(*ch)) {
                format_options.align = Some(FormatAlign::new(alignment, Some(ch)));
                str_iter.next();
            }
        }
    }

    parse_from_sign(
        format_options.align.is_none().then_some((ch_index, ch)),
        offset,
        initial_str,
        str_iter,
        format_options,
    )
}

fn parse_from_sign<'a>(
    prev_char: Option<(usize, char)>,
    offset: usize,
    initial_str: &'a str,
    str_iter: &mut StrIter<'a>,
    format_options: &mut FormatOptions<'a>,
) -> Result<(), ParseError> {
    let Some((ch_index, ch)) = prev_char.or_else(|| str_iter.next()) else {
        return Ok(());
    };

    match ch {
        '+' => format_options.sign = Some(Sign::Plus),
        '-' => format_options.sign = Some(Sign::Minus),
        _ => {}
    }

    parse_from_alternate_form(
        format_options.sign.is_none().then_some((ch_index, ch)),
        offset,
        initial_str,
        str_iter,
        format_options,
    )
}

fn parse_from_alternate_form<'a>(
    prev_char: Option<(usize, char)>,
    offset: usize,
    initial_str: &'a str,
    str_iter: &mut StrIter<'a>,
    format_options: &mut FormatOptions<'a>,
) -> Result<(), ParseError> {
    let Some((ch_index, ch)) = prev_char.or_else(|| str_iter.next()) else {
        return Ok(());
    };

    if ch == '#' {
        format_options.use_alternate_form = true;
    }

    parse_from_zero_padding(
        (!format_options.use_alternate_form).then_some((ch_index, ch)),
        offset,
        initial_str,
        str_iter,
        format_options,
    )
}

fn parse_from_zero_padding<'a>(
    prev_char: Option<(usize, char)>,
    offset: usize,
    initial_str: &'a str,
    str_iter: &mut StrIter<'a>,
    format_options: &mut FormatOptions<'a>,
) -> Result<(), ParseError> {
    let Some((ch_index, ch)) = prev_char.or_else(|| str_iter.next()) else {
        return Ok(());
    };

    if ch == '0' {
        format_options.use_zero_padding = true;
    }

    parse_from_width(
        (!format_options.use_zero_padding).then_some((ch_index, ch)),
        offset,
        initial_str,
        str_iter,
        format_options,
    )
}

fn parse_from_width<'a>(
    prev_char: Option<(usize, char)>,
    offset: usize,
    initial_str: &'a str,
    str_iter: &mut StrIter<'a>,
    format_options: &mut FormatOptions<'a>,
) -> Result<(), ParseError> {
    let Some((ch_index, ch)) = prev_char.or_else(|| str_iter.next()) else {
        return Ok(());
    };

    let has_width_argument = has_width_count_argument(initial_str);

    if has_width_argument && ch == '$' && format_options.use_zero_padding {
        format_options.use_zero_padding = false;
        format_options.width = Some(FormatCount::Argument(Argument::Index(Integer::new(0))));
    } else if ch.is_ascii_digit() || has_width_argument {
        format_options.width = Some(parse_count(offset, ch, ch_index, initial_str, str_iter)?);
    }

    return parse_from_precision(
        format_options.width.is_none().then_some((ch_index, ch)),
        offset,
        initial_str,
        str_iter,
        format_options,
    );

    fn has_width_count_argument(str: &str) -> bool {
        match (str.find('$'), str.find('.')) {
            (Some(argument_end_index), Some(precision_start_index)) => argument_end_index < precision_start_index,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => false,
        }
    }
}

fn parse_from_precision<'a>(
    prev_char: Option<(usize, char)>,
    offset: usize,
    initial_str: &'a str,
    str_iter: &mut StrIter<'a>,
    format_options: &mut FormatOptions<'a>,
) -> Result<(), ParseError> {
    let Some((ch_index, ch)) = prev_char.or_else(|| str_iter.next()) else {
        return Ok(());
    };

    if ch == '.' {
        format_options.precision = Some(parse_precision(offset, ch_index, initial_str, str_iter)?);
    }

    return parse_from_format_trait(
        format_options.precision.is_none().then_some((ch_index, ch)),
        initial_str,
        str_iter,
        format_options,
    );

    fn parse_precision<'a>(
        offset: usize,
        dot_index: usize,
        initial_str: &'a str,
        str_iter: &mut StrIter<'a>,
    ) -> Result<FormatPrecision<'a>, ParseError> {
        let Some((first_char_index, first_char)) = str_iter.next() else {
            return Err(ParseError::new_char(
                offset + dot_index,
                FormatPrecisionParseError::Empty,
            ));
        };

        if first_char == '*' {
            return Ok(FormatPrecision::NextArgument);
        }

        parse_count(offset, first_char, first_char_index, initial_str, str_iter).map(FormatPrecision::Count)
    }
}

fn parse_from_format_trait(
    prev_char: Option<(usize, char)>,
    initial_str: &str,
    str_iter: &mut StrIter,
    format_options: &mut FormatOptions,
) -> Result<(), ParseError> {
    let Some((ch_index, _)) = prev_char.or_else(|| str_iter.next()) else {
        return Ok(());
    };

    let format_trait_str = &initial_str[ch_index..];
    format_options.format_trait = FormatTrait::parse(ch_index, format_trait_str)?;

    Ok(())
}

fn parse_count<'a>(
    offset: usize,
    first_char: char,
    first_char_index: usize,
    initial_str: &'a str,
    str_iter: &mut StrIter<'a>,
) -> Result<FormatCount<'a>, ParseError> {
    // count integer or count index argument
    if let Some(first_digit) = first_char.to_digit(10) {
        let mut number = first_digit;
        let mut power = 0u32;

        while let Some((_, next_char)) = str_iter.peek() {
            if let Some(digit) = next_char.to_digit(10) {
                number += digit * 10u32.pow(power);
                power += 1;

                str_iter.next();
                continue;
            }

            if *next_char == '$' {
                str_iter.next();

                return Ok(FormatCount::Argument(Argument::Index(Integer::new(number as usize))));
            } else {
                return Ok(FormatCount::Integer(Integer::new(number as usize)));
            }
        }

        // Reached end of string
        Ok(FormatCount::Integer(Integer::new(number as usize)))
    }
    // count named argument
    else {
        match str_iter.find(|(_, ch)| *ch == '$') {
            Some((end_index, _)) => {
                let identifier_str = &initial_str[first_char_index..end_index];
                let identifier = Identifier::parse(first_char_index, identifier_str)?;

                Ok(FormatCount::Argument(Argument::Identifier(identifier)))
            }
            None => Err(ParseError::new(
                offset,
                first_char_index..initial_str.len(),
                FormatCountParseError::UnclosedArgument,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        assert_format_options("", Default::default());
    }

    #[test]
    fn parse_alignment() {
        let expected = FormatOptions::builder()
            .align(FormatAlign::new(Alignment::Left, None))
            .build();

        assert_format_options("<", expected);
    }

    #[test]
    fn parse_fill_alignment() {
        let expected = FormatOptions::builder()
            .align(FormatAlign::new(Alignment::Center, Some('🦀')))
            .build();

        assert_format_options("🦀^", expected);
    }

    #[test]
    fn parse_plus_sign() {
        let expected = FormatOptions::builder().sign(Sign::Plus).build();
        assert_format_options("+", expected);
    }

    #[test]
    fn parse_minus_sign() {
        let expected = FormatOptions::builder().sign(Sign::Minus).build();
        assert_format_options("-", expected);
    }

    #[test]
    fn parse_alternate_form() {
        let expected = FormatOptions::builder().use_alternate_form(true).build();
        assert_format_options("#", expected);
    }

    #[test]
    fn parse_zero_padding() {
        let expected = FormatOptions::builder().use_zero_padding(true).build();
        assert_format_options("0", expected);
    }

    #[test]
    fn parse_width_count_literal() {
        let expected = FormatOptions::builder()
            .width(FormatCount::Integer(Integer::new(1)))
            .build();
        assert_format_options("1", expected);
    }

    #[test]
    fn parse_width_count_index_argument() {
        let expected = FormatOptions::builder()
            .width(FormatCount::Argument(Argument::Index(Integer::new(1))))
            .build();

        assert_format_options("1$", expected);
    }

    // assert that this isn't parsed as zero padding with an unclosed width argument
    #[test]
    fn parse_width_count_zero_index_argument() {
        let count = FormatCount::Argument(Argument::Index(Integer::new(0)));

        let expected = FormatOptions::builder().width(count).build();
        assert_format_options("0$", expected);

        let expected = FormatOptions::builder().use_zero_padding(true).width(count).build();
        assert_format_options("00$", expected);
    }

    #[test]
    fn parse_width_count_named_argument() {
        let identifier = Identifier::parse(0, "x").unwrap();

        let expected = FormatOptions::builder()
            .width(FormatCount::Argument(Argument::Identifier(identifier)))
            .build();

        assert_format_options("x$", expected);
    }

    #[test]
    fn parse_precision_next_argument() {
        let expected = FormatOptions::builder()
            .precision(FormatPrecision::NextArgument)
            .build();
        assert_format_options(".*", expected);
    }

    #[test]
    fn parse_precision_count_literal() {
        let expected = FormatOptions::builder()
            .precision(FormatPrecision::Count(FormatCount::Integer(Integer::new(1))))
            .build();
        assert_format_options(".01", expected);
    }

    #[test]
    fn parse_format_trait() {
        let expected = FormatOptions::builder()
            .format_trait(FormatTrait::DebugLowerHex)
            .build();
        assert_format_options("x?", expected);
    }

    #[test]
    fn parse_all_combined() {
        let identifier = Identifier::parse(0, "x").unwrap();
        let count = FormatCount::Argument(Argument::Identifier(identifier));

        let expected = FormatOptions::builder()
            .align(FormatAlign::new(Alignment::Right, Some('🦀')))
            .sign(Sign::Plus)
            .use_alternate_form(true)
            .use_zero_padding(true)
            .width(count)
            .precision(FormatPrecision::Count(count))
            .format_trait(FormatTrait::DebugLowerHex)
            .build();

        assert_format_options("🦀>+#0x$.x$x?", expected);
    }

    #[test]
    fn precision_empty_error() {
        let expected = ParseError::new_char(2, FormatPrecisionParseError::Empty);
        let actual = FormatOptions::parse(0, "00.").unwrap_err();

        assert_eq!(expected, actual);
    }

    #[test]
    fn count_eof_error() {
        let expected = ParseError::new(3, 0..3, FormatCountParseError::UnclosedArgument);
        let actual = FormatOptions::parse(0, "00.xxx").unwrap_err();

        assert_eq!(expected, actual);
    }

    fn assert_format_options(format_options_str: &str, expected: FormatOptions) {
        let actual = FormatOptions::parse(0, format_options_str).unwrap();
        assert_eq!(expected, actual);
    }
}
