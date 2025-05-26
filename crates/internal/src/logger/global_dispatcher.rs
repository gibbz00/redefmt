use core::sync::atomic::{AtomicU8, Ordering};

use critical_section::{CriticalSection, Mutex};

use crate::*;

enum GlobalDispatcherKind {
    Static(&'static mut dyn Dispatcher),
    // IMPROVEMENT: use unsafe cell?
    #[cfg(feature = "alloc")]
    Boxed(Mutex<core::cell::RefCell<alloc::boxed::Box<dyn Dispatcher + Sync + Send>>>),
}

static mut GLOBAL_DISPATCHER: Option<GlobalDispatcherKind> = None;

static GLOBAL_DISPATCHER_STATE: AtomicU8 = AtomicU8::new(UNINITIALIZED);

const UNINITIALIZED: u8 = 0;
const INITIALIZING: u8 = 1;
const INITIALIZED: u8 = 2;

pub struct GlobalDispatcher;

impl GlobalDispatcher {
    #[cfg(feature = "alloc")]
    pub fn init_alloc(dispatcher: impl Dispatcher + Send + Sync + 'static) -> Result<(), GlobalLoggerError> {
        let kind =
            GlobalDispatcherKind::Boxed(Mutex::new(core::cell::RefCell::new(alloc::boxed::Box::new(dispatcher))));
        Self::init_impl(kind)
    }

    pub fn init_static(dispatcher: &'static mut dyn Dispatcher) -> Result<(), GlobalLoggerError> {
        let kind = GlobalDispatcherKind::Static(dispatcher);
        Self::init_impl(kind)
    }

    fn init_impl(dispatcher: GlobalDispatcherKind) -> Result<(), GlobalLoggerError> {
        GLOBAL_DISPATCHER_STATE
            .compare_exchange(UNINITIALIZED, INITIALIZING, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| GlobalLoggerError::LoggerAlreadyInitialized)?;

        // SAFETY: static not yet initialized, check and initializing flag is
        // atomic and sequentially consistent
        unsafe { GLOBAL_DISPATCHER = Some(dispatcher) };

        GLOBAL_DISPATCHER_STATE.store(INITIALIZED, Ordering::SeqCst);

        Ok(())
    }

    // TODO: document how this function is locking to ensure that statement writes don't get
    // intertwined, and how write no ops are done if none is set
    pub(crate) fn global_dispatcher() -> GlobalDispatcherHandle {
        match GLOBAL_DISPATCHER_STATE.load(Ordering::Acquire) == INITIALIZED {
            true => {
                let restore_state = unsafe { critical_section::acquire() };

                // SAFETY: cs_token created after a `critical_section::acquire` call
                let cs_token = unsafe { critical_section::CriticalSection::new() };

                let inner = GlobalDispatcherHandleInner { restore_state, cs_token };

                GlobalDispatcherHandle { inner: Some(inner) }
            }
            false => GlobalDispatcherHandle { inner: None },
        }
    }
}

pub struct GlobalDispatcherHandle {
    inner: Option<GlobalDispatcherHandleInner>,
}

pub struct GlobalDispatcherHandleInner {
    restore_state: critical_section::RestoreState,
    cs_token: CriticalSection<'static>,
}

impl GlobalDispatcherHandle {
    pub fn get(&mut self, f: impl FnOnce(&mut dyn Dispatcher)) {
        let Some(handle) = &self.inner else {
            return;
        };

        // SAFETY:
        // Inner handle only set to `Option::Some` if in initialized state, and initialized state is
        // only set once. Static mut ref therefore should therefore be sound.
        let dispatcher_ref = unsafe {
            #[allow(static_mut_refs)]
            GLOBAL_DISPATCHER.as_mut()
        }
        .expect("handle set when global dispatcher was not set");

        match dispatcher_ref {
            GlobalDispatcherKind::Static(dispatcher) => {
                f(*dispatcher);
            }
            #[cfg(feature = "alloc")]
            GlobalDispatcherKind::Boxed(dispatcher_mutex) => {
                // - called within a critical section for the boxed variant
                let mut dispatcher_handle = dispatcher_mutex.borrow(handle.cs_token).borrow_mut();
                f(dispatcher_handle.as_mut());
            }
        }
    }
}

impl Drop for GlobalDispatcherHandle {
    fn drop(&mut self) {
        if let Some(handle) = &self.inner {
            // SAFETY: restore state received the corresponding call to `critical_sectino::acquire`
            unsafe { critical_section::release(handle.restore_state) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_get() {
        let bytes = [1, 2, 3];
        let shared_dispatcher = SharedTestDispatcher::new();

        GlobalDispatcher::init_alloc(shared_dispatcher.clone()).unwrap();

        shared_dispatcher.assert_bytes(&[]);

        GlobalDispatcher::global_dispatcher().get(|dispatcher| dispatcher.write(&bytes));

        shared_dispatcher.assert_bytes(&bytes);

        assert!(GlobalDispatcher::init_alloc(shared_dispatcher).is_err());
    }
}
