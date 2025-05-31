mod core {
    use crate::*;

    pub struct PrinterConfig {
        stamp: PrintStampConfig,
    }
}
pub use core::PrinterConfig;

pub mod stamp;
pub(crate) use stamp::*;
