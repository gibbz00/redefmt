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
    use redefmt_args::MappedFormatExpression;

    // TODO: integrate with thiserror
    #[derive(Debug, PartialEq)]
    pub enum CounterNewError {
        /// Counter format string expects one and only one provided argument.
        ArgCount,
    }

    #[derive(Debug)]
    pub struct Counter {
        format_expression: MappedFormatExpression<'static>,
    }

    impl Counter {
        pub fn new(format_expression: MappedFormatExpression<'static>) -> Result<Self, CounterNewError> {
            if format_expression.provided_args().count() != 1 {
                return Err(CounterNewError::ArgCount);
            }

            Ok(Counter { format_expression })
        }
    }
}
pub(crate) use counter::{Counter, CounterNewError};
