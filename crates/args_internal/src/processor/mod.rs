mod internal;
pub(crate) use internal::FormatProcessor;

mod error;
pub use error::ProcessorError;

mod config;
pub use config::ProcessorConfig;

mod arg_capturer;
pub use arg_capturer::ArgCapturer;
#[cfg(feature = "syn")]
pub(crate) use arg_capturer::SynArgCapturer;
