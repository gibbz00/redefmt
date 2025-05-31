use crate::*;

pub enum PrintStampConfig {
    Counter(PrintCounterConfig),
    // Offset from datetime of first frame
    OffsetTimestamp(PrintTimestampConfig),
    UnixTimestamp(PrintTimestampConfig),
}
