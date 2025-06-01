mod core;
pub use core::FormatProcessor;

mod string;
pub use string::ProcessedFormatString;

mod error;
pub use error::ProcessorError;

mod config;
pub use config::ProcessorConfig;

mod arg_capturer;
pub use arg_capturer::ArgCapturer;
#[cfg(feature = "syn")]
pub(crate) use arg_capturer::SynArgCapturer;
