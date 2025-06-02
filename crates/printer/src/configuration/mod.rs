mod core;
pub use core::{PrettyPrinterConfig, PrettyPrinterConfigError};

pub mod stamp;
pub(crate) use stamp::{PrintStampConfig, PrintTimestampConfig, PrintTimestampPrecisionConfig};
