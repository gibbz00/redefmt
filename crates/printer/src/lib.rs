//! # `redefmt-printer`

mod configuration {
    mod stamp {
        use chrono::{Utc, format::DelayedFormat};
        use redefmt_args::FormatString;

        pub enum Stamp {
            Counter(Counter),
            // Offset from datetime of first frame
            OffsetTimestamp(Timestamp),
            UnixTimestamp(Timestamp),
        }

        pub struct Counter {
            // TODO: FormatString may only require one argument
            format_string: FormatString<'static>,
        }

        pub struct Timestamp {
            precision: TimestampPrecision,
            time_format_string: DelayedFormat<Utc>,
        }

        pub enum TimestampPrecision {
            Microseconds,
            Milliseconds,
        }
    }
}
