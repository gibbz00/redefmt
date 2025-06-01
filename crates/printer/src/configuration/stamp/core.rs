use crate::*;

pub enum PrintStampConfig {
    Counter,
    // Offset from datetime of first frame
    OffsetTimestamp(PrintTimestampConfig),
    UnixTimestamp(PrintTimestampConfig),
}
