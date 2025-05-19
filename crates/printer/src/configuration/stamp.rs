mod core {
    use crate::*;

    pub enum Stamp {
        Counter(Counter),
        // Offset from datetime of first frame
        OffsetTimestamp(Timestamp),
        UnixTimestamp(Timestamp),
    }
}

mod timestamp {
    use chrono::{Utc, format::DelayedFormat};

    pub struct Timestamp {
        precision: TimestampPrecision,
        time_format_string: DelayedFormat<Utc>,
    }

    pub enum TimestampPrecision {
        Microseconds,
        Milliseconds,
    }
}
pub(crate) use timestamp::{Timestamp, TimestampPrecision};

mod counter {
    use redefmt_args::{FormatString, ParseError, RequiredArgsError};

    // TODO: integrate with thiserror
    #[derive(Debug, PartialEq)]
    pub enum CounterNewError {
        Format(ParseError),
        Args(RequiredArgsError),
        /// Counter format string expects one and only one unnamed argument.
        ArgCount,
    }

    #[derive(Debug)]
    pub struct Counter {
        format_string: FormatString<'static>,
    }

    impl Counter {
        pub fn new(format_string: &str) -> Result<Self, CounterNewError> {
            let format_string = FormatString::parse(format_string)
                .map_err(CounterNewError::Format)?
                .owned();

            let required_arguments = format_string.required_arguments().map_err(CounterNewError::Args)?;

            if !required_arguments.named_arguments().is_empty() || *required_arguments.unnamed_argument_count() != 1 {
                return Err(CounterNewError::ArgCount);
            }

            drop(required_arguments);

            Ok(Counter { format_string })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn new() {
            assert!(Counter::new("counter: {}").is_ok());
        }

        #[test]
        fn new_unnamed_arg_err() {
            let expected_error = CounterNewError::ArgCount;
            let actual_error = Counter::new("counter: {} {}").unwrap_err();

            assert_eq!(expected_error, actual_error);
        }

        #[test]
        fn new_named_arg_err() {
            let expected_error = CounterNewError::ArgCount;
            let actual_error = Counter::new("counter: {counter}").unwrap_err();

            assert_eq!(expected_error, actual_error);
        }
    }
}
pub(crate) use counter::{Counter, CounterNewError};
