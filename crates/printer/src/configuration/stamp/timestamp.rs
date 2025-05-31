use chrono::{Utc, format::DelayedFormat};

pub struct PrintTimestampConfig {
    precision: PrintTimestampPrecisionConfig,
    time_format_string: DelayedFormat<Utc>,
}

pub enum PrintTimestampPrecisionConfig {
    Microseconds,
    Milliseconds,
}
