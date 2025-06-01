mod core {
    use crate::*;

    pub struct PrinterConfig {
        pub stamp: PrintStampConfig,
    }
}
pub use core::PrinterConfig;

pub mod stamp;
pub(crate) use stamp::*;
