mod internal;
pub(crate) use internal::InternalArgumentResolver;

mod error;
pub use error::ResolveArgsError;

mod config;
pub use config::ResolverConfig;

mod arg_capturer;
pub use arg_capturer::ArgCapturer;
#[cfg(feature = "syn")]
pub(crate) use arg_capturer::SynArgCapturer;
