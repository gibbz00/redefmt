mod core {
    pub trait Dispatcher {
        fn write(&mut self, bytes: &[u8]);
    }
}
pub(crate) use core::Dispatcher;

#[cfg(test)]
pub(crate) use test_dispatcher::{SharedTestDispatcher, SimpleTestDispatcher};
#[cfg(test)]
mod test_dispatcher {
    use alloc::sync::Arc;
    use core::cell::RefCell;

    use bytes::{BufMut, BytesMut};
    use critical_section::Mutex;

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

    #[derive(Clone)]
    pub struct SharedTestDispatcher {
        bytes: Arc<Mutex<RefCell<BytesMut>>>,
    }

    impl SharedTestDispatcher {
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
    }

    impl Dispatcher for SharedTestDispatcher {
        fn write(&mut self, bytes: &[u8]) {
            critical_section::with(|cs| {
                self.bytes.borrow(cs).borrow_mut().put_slice(bytes);
            })
        }
    }
}
