#[cfg(feature = "syn")]
mod provided;
#[cfg(feature = "syn")]
pub use provided::ProvidedArgs;

mod mapping;
pub use mapping::ProvidedArgsMapping;

#[cfg(feature = "syn")]
mod resolver;
#[cfg(feature = "syn")]
pub(crate) use resolver::{ArgumentResolver, ResolveArgsError};
