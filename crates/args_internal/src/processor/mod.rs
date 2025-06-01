mod core;
pub use core::FormatProcessor;

mod provided_args;
pub(crate) use provided_args::ProvidedStaticArgs;

mod processed;
pub use processed::ProcessedFormatString;

mod error;
pub use error::ProcessorError;

mod config;
pub use config::{DynamicProcessorConfig, StaticProcessorConfig};

mod arg_capturer;
pub use arg_capturer::ArgCapturer;
#[cfg(feature = "syn")]
pub(crate) use arg_capturer::SynArgCapturer;
