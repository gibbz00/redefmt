mod core;
pub use core::FormatProcessor;

mod provided_args;
pub(crate) use provided_args::ProvidedStaticArgs;

mod string;
pub use string::ProcessedFormatString;

mod error;
pub use error::FormatProcessorError;

mod config;
pub use config::{DynamicProcessorConfig, StaticProcessorConfig};

mod arg_capturer;
pub use arg_capturer::ArgCapturer;
#[cfg(feature = "syn")]
pub(crate) use arg_capturer::SynArgCapturer;
