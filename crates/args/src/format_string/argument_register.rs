#[derive(Debug, PartialEq, derive_getters::Getters)]
pub struct RequiredArgs<'a, 's> {
    pub(crate) unnamed_argument_count: usize,
    pub(crate) named_arguments: HashSet<&'s Identifier<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum RequiredArgsError {
    /// Provided index larger than unique argument count
    InvalidIndex,
}

use core::cmp::Ordering;

use bit_set::BitSet;
use hashbrown::HashSet;

use crate::*;

#[derive(Default)]
pub struct ArgumentRegister<'a, 's> {
    unnamed_argument_count: usize,
    named_arguments: HashSet<&'s Identifier<'a>>,
    index_arguments: BitSet,
}

impl<'a, 's> ArgumentRegister<'a, 's> {
    pub(crate) fn new(format_string: &'s FormatString<'a>) -> Self {
        let mut register = ArgumentRegister::default();

        for segment in &format_string.segments {
            if let FormatStringSegment::Format(format_segment) = segment {
                match format_segment.argument() {
                    Some(argument) => {
                        register.register_argument(argument);
                    }
                    None => {
                        register.unnamed_argument_count += 1;
                    }
                }

                // next argument requirements?
                if let Some(FormatPrecision::Count(FormatCount::Argument(precision_argument))) =
                    format_segment.options().precision()
                {
                    register.register_argument(precision_argument);
                }

                if let Some(FormatCount::Argument(width_argument)) = format_segment.options().width() {
                    register.register_argument(width_argument);
                }
            }
        }

        register
    }

    fn register_argument(&mut self, argument: &'s FormatArgument<'a>) {
        match argument {
            FormatArgument::Index(integer) => self.index_arguments.insert(integer.inner()),
            FormatArgument::Identifier(identifier) => self.named_arguments.insert(identifier),
        };
    }

    pub(crate) fn resolve(self) -> Result<RequiredArgs<'a, 's>, RequiredArgsError> {
        match self.index_arguments.is_empty() {
            true => self.resolve_unindexed(),
            false => self.resolve_indexed(),
        }
    }

    fn resolve_unindexed(self) -> Result<RequiredArgs<'a, 's>, RequiredArgsError> {
        let ArgumentRegister { unnamed_argument_count, named_arguments, .. } = self;
        Ok(RequiredArgs { unnamed_argument_count, named_arguments })
    }

    fn resolve_indexed(self) -> Result<RequiredArgs<'a, 's>, RequiredArgsError> {
        let max_index = self.max_index();
        let named_argument_count = self.named_arguments.len();
        let unnamed_argument_count = self.unnamed_argument_count;

        let non_indexed_argument_count = named_argument_count + unnamed_argument_count;

        let count = match max_index.cmp(&non_indexed_argument_count) {
            Ordering::Less => unnamed_argument_count,
            Ordering::Equal => unnamed_argument_count + 1,
            Ordering::Greater => {
                self.assert_continuous_from(non_indexed_argument_count)?;

                max_index + 1 - non_indexed_argument_count
            }
        };

        Ok(RequiredArgs { unnamed_argument_count: count, named_arguments: self.named_arguments })
    }

    /// panics if index is out of bounds
    fn assert_continuous_from(&self, index: usize) -> Result<usize, RequiredArgsError> {
        let max_index = self.max_index();
        for i in index..max_index + 1 {
            if !self.index_arguments.contains(i) {
                return Err(RequiredArgsError::InvalidIndex);
            }
        }

        Ok(max_index)
    }

    /// panics if the index_arguments is empty
    fn max_index(&self) -> usize {
        // self.index_argument.insert() grows with `false` padding
        let mut maybe_max_index = self.index_arguments.get_ref().len() - 1;

        loop {
            match self.index_arguments.contains(maybe_max_index) {
                true => return maybe_max_index,
                false => {
                    maybe_max_index -= 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn empty() {
        assert_required("", 0, []);
    }

    #[test]
    fn positional() {
        assert_required("{}", 1, []);
        assert_required("{} {}", 2, []);
    }

    #[test]
    fn indexed() {
        assert_required("{0}", 1, []);
    }

    #[test]
    fn index_reuse() {
        assert_required("{0} {0}", 1, []);
    }

    #[test]
    fn index_unique() {
        assert_required("{0} {1}", 2, []);
    }

    #[test]
    fn named() {
        assert_required("{x}", 0, ["x"]);
    }

    #[test]
    fn named_unique() {
        assert_required("{x} {y}", 0, ["x", "y"]);
    }

    #[test]
    fn named_reuse() {
        assert_required("{x} {x}", 0, ["x"]);
    }

    #[test]
    fn positional_with_index_reuse() {
        assert_required("{} {0}", 1, []);
    }

    #[test]
    fn positional_with_index_unique() {
        assert_required("{} {1}", 2, []);
    }

    #[test]
    fn positional_with_named() {
        assert_required("{x} {}", 1, ["x"]);
    }

    #[test]
    fn indexed_with_named_reuse() {
        assert_required("{0} {x}", 0, ["x"]);
    }

    #[test]
    fn all_combined() {
        assert_required("{1} {x} {}", 1, ["x"]);
        assert_required("{2} {x} {}", 2, ["x"]);
    }

    #[test]
    fn registers_precision() {
        assert_required("{:.0$}", 1, []);
        assert_required("{:.1$}", 2, []);
        assert_required("{x:.0$}", 0, ["x"]);
        assert_required("{x:.1$}", 1, ["x"]);
    }

    #[test]
    fn registers_width() {
        assert_required("{:0$}", 1, []);
        assert_required("{:1$}", 2, []);
        assert_required("{x:0$}", 0, ["x"]);
        assert_required("{x:1$}", 1, ["x"]);
    }

    #[test]
    fn invalid_index_error() {
        assert_invalid("{1}");
        assert_invalid("{} {2}");
        assert_invalid("{x} {2}");
        assert_invalid("{x} {} {3}");
        assert_invalid("{x} {} {2} {5}");
    }

    #[test]
    fn discontinuous_from() {
        let format_string = FormatString::parse("{0} {2} {3} {4}").unwrap();
        let argument_register = ArgumentRegister::new(&format_string);

        assert_discountinous(&argument_register, 0);
        assert_discountinous(&argument_register, 1);
        assert!(argument_register.assert_continuous_from(2).is_ok());
        assert!(argument_register.assert_continuous_from(4).is_ok());

        fn assert_discountinous(argument_register: &ArgumentRegister, from: usize) {
            let discontinuous_error = argument_register.assert_continuous_from(from).unwrap_err();
            assert_eq!(RequiredArgsError::InvalidIndex, discontinuous_error);
        }
    }

    fn assert_required<'a>(str: &str, expected_unique: usize, expected_named_arguments: impl AsRef<[&'a str]>) {
        let expected_named_arguments = expected_named_arguments
            .as_ref()
            .iter()
            .map(|str| Identifier::parse(0, *str).unwrap())
            .collect::<Vec<_>>();

        let expected = RequiredArgs {
            unnamed_argument_count: expected_unique,
            named_arguments: expected_named_arguments.iter().collect(),
        };

        let format_string = FormatString::parse(str).unwrap();
        let actual = format_string.required_arguments().unwrap();

        assert_eq!(expected, actual);
    }

    fn assert_invalid(invalid_str: &str) {
        let actual = FormatString::parse(invalid_str)
            .unwrap()
            .required_arguments()
            .unwrap_err();

        assert_eq!(RequiredArgsError::InvalidIndex, actual);
    }
}
