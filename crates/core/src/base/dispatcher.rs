pub trait Dispatcher {
    fn write(&mut self, bytes: &[u8]);
}

#[cfg(test)]
mod crate_test_dispatcher {
    use crate::*;

    pub struct NoopTestDispatcher;

    impl Dispatcher for NoopTestDispatcher {
        fn write(&mut self, _bytes: &[u8]) {}
    }
}
#[cfg(test)]
pub use crate_test_dispatcher::NoopTestDispatcher;

#[cfg(feature = "base-testing")]
mod simple_dispatcher {
    use bytes::{BufMut, BytesMut};

    use crate::*;

    #[derive(Default)]
    pub struct SimpleTestDispatcher {
        pub bytes: BytesMut,
    }

    impl Dispatcher for SimpleTestDispatcher {
        fn write(&mut self, bytes: &[u8]) {
            self.bytes.put_slice(bytes);
        }
    }
}
#[cfg(feature = "base-testing")]
pub use simple_dispatcher::SimpleTestDispatcher;

#[cfg(feature = "base-testing")]
mod shared_dispatcher {
    use alloc::sync::Arc;
    use core::cell::RefCell;

    use bytes::{BufMut, BytesMut};
    use critical_section::Mutex;

    use crate::*;

    #[derive(Clone)]
    pub struct SharedTestDispatcher {
        bytes: Arc<Mutex<RefCell<BytesMut>>>,
    }

    impl SharedTestDispatcher {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self { bytes: Arc::new(Mutex::new(RefCell::new(BytesMut::new()))) }
        }

        pub fn assert_bytes(&self, expected_bytes: &[u8]) {
            critical_section::with(|cs| {
                let bytes_ref = self.bytes.borrow_ref(cs);
                let actual_bytes = bytes_ref.as_ref();
                assert_eq!(expected_bytes, actual_bytes);
            })
        }

        pub fn take_bytes(&self) -> BytesMut {
            let mut bytes = BytesMut::new();

            critical_section::with(|cs| {
                let mut bytes_muf_ref = self.bytes.borrow_ref_mut(cs);
                core::mem::swap(&mut bytes, &mut bytes_muf_ref);
            });

            bytes
        }
    }

    impl Dispatcher for SharedTestDispatcher {
        fn write(&mut self, bytes: &[u8]) {
            critical_section::with(|cs| {
                self.bytes.borrow(cs).borrow_mut().put_slice(bytes);
            })
        }
    }
}
#[cfg(feature = "base-testing")]
pub use shared_dispatcher::SharedTestDispatcher;
