mod dispatcher;
pub use dispatcher::Dispatcher;
#[doc(hidden)]
#[cfg(feature = "testing")]
pub use dispatcher::{SharedTestDispatcher, SimpleTestDispatcher};
