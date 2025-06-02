pub enum PrintStampConfig {
    Counter,
    /// Offset from datetime of first frame
    OffsetTimestamp(PrintTimestampConfig),
    UnixTimestamp(PrintTimestampConfig),
}

pub struct PrintTimestampConfig {
    pub timestamp_precision: PrintTimestampPrecisionConfig,
    /// `strftime` inspired string passed to `chrono`'s [`DateTime::format`][fmt]
    ///
    /// Defaults to RFC 3339 / ISO 8601 if none is provided.
    ///
    /// [fmt]: https://docs.rs/chrono/0.4.41/chrono/struct.DateTime.html#method.format
    pub datetime_format_string: Option<String>,
}

pub enum PrintTimestampPrecisionConfig {
    Microseconds,
    Milliseconds,
}
