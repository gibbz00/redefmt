#[cfg(feature = "syn")]
mod provided;
#[cfg(feature = "syn")]
pub use provided::ProvidedArgs;

mod deferred;
pub use deferred::DeferredArgs;

#[cfg(feature = "syn")]
mod resolver;
#[cfg(feature = "syn")]
pub(crate) use resolver::{ArgumentResolver, ResolveArgsError};
