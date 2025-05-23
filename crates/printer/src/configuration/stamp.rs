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
    use redefmt_args::{FormatString, FormatStringParseError};

    // TODO: integrate with thiserror
    #[derive(Debug, PartialEq)]
    pub enum CounterNewError {
        Format(FormatStringParseError),
        /// Counter format string expects one and only one unnamed argument.
        ArgCount,
    }

    #[derive(Debug)]
    pub struct Counter {
        format_string: FormatString<'static>,
    }

    impl Counter {
        pub fn new(format_string: &str) -> Result<Self, CounterNewError> {
            let mut format_string = FormatString::parse(format_string)
                .map_err(CounterNewError::Format)?
                .owned();

            // TEMP:
            panic!("fixme: format string validation should accept only one argument, which may be named counter");

            Ok(Counter { format_string })
        }
    }
}
pub(crate) use counter::{Counter, CounterNewError};
