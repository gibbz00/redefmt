use alloc::boxed::Box;
use core::cell::RefCell;

use critical_section::{CriticalSection, Mutex};

use crate::*;

static mut GLOBAL_DISPATCHER: Option<critical_section::Mutex<RefCell<Box<dyn Dispatcher + Sync + Send>>>> = None;

pub struct GlobalRegistry;

impl GlobalRegistry {
    pub fn init(dispatcher: impl Dispatcher + Send + Sync + 'static) {
        // TODO: panic if called twice
        critical_section::with(|_cs| {
            // SAFETY: within critical section
            unsafe { GLOBAL_DISPATCHER = Some(Mutex::new(RefCell::new(Box::new(dispatcher)))) };
        })
    }

    // TODO: document how this function is locking
    // TODO: panic rather than dead lock if called again before dispatcher handle is released?
    pub(crate) fn dispatcher() -> DispatcherHandle {
        let restore_state = unsafe { critical_section::acquire() };

        // SAFETY: cs_token created after a `critical_section::acquire` call
        let cs_token = unsafe { critical_section::CriticalSection::new() };

        DispatcherHandle { restore_state, cs_token }
    }

    pub(crate) fn stamper() -> Option<&'static dyn Stamper> {
        todo!()
    }
}

pub struct DispatcherHandle {
    restore_state: critical_section::RestoreState,
    cs_token: CriticalSection<'static>,
}

impl DispatcherHandle {
    pub fn write(&mut self, bytes: &[u8]) {
        // SAFETY: called within a critical section
        #[allow(static_mut_refs)]
        let dispatcher_ref = unsafe { GLOBAL_DISPATCHER.as_ref().unwrap() };

        let mut dispatcher_handle = dispatcher_ref.borrow(self.cs_token).borrow_mut();
        dispatcher_handle.write(bytes);
    }
}

impl Drop for DispatcherHandle {
    fn drop(&mut self) {
        // SAFETY: restore state received the corresponding call to `critical_sectino::acquire`
        unsafe { critical_section::release(self.restore_state) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_get() {
        let bytes = [1, 2, 3];
        let shared_dispatcher = SharedTestDispatcher::new();

        GlobalRegistry::init(shared_dispatcher.clone());

        shared_dispatcher.assert_bytes(&[]);

        GlobalRegistry::dispatcher().write(&bytes);

        shared_dispatcher.assert_bytes(&bytes);
    }
}
