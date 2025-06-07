use core::sync::atomic::{AtomicU8, Ordering};

use crate::*;

static mut GLOBAL_DISPATCHER: Option<&'static mut dyn Dispatcher> = None;

static GLOBAL_DISPATCHER_STATE: AtomicU8 = AtomicU8::new(UNINITIALIZED);

const UNINITIALIZED: u8 = 0;
const INITIALIZING: u8 = 1;
const INITIALIZED: u8 = 2;

pub struct GlobalDispatcher;

impl GlobalDispatcher {
    #[cfg(feature = "alloc")]
    pub fn init_boxed(dispatcher: impl Dispatcher + Send + Sync + 'static) -> Result<(), GlobalLoggerError> {
        use alloc::boxed::Box;
        let leaked_dispatcher = Box::leak(Box::new(dispatcher));
        Self::init_static(leaked_dispatcher)
    }

    pub fn init_static(dispatcher: &'static mut dyn Dispatcher) -> Result<(), GlobalLoggerError> {
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

                GlobalDispatcherHandle { restore_state: Some(restore_state) }
            }
            false => GlobalDispatcherHandle { restore_state: None },
        }
    }
}

pub struct GlobalDispatcherHandle {
    restore_state: Option<critical_section::RestoreState>,
}

impl GlobalDispatcherHandle {
    pub fn get(&mut self, f: impl FnOnce(&mut dyn Dispatcher)) {
        // SAFETY:
        // Inner handle only set to `Option::Some` if in initialized state, and initialized state is
        // only set once. Static mut ref should therefore be sound.
        let maybe_dispatcher = unsafe {
            #[allow(static_mut_refs)]
            GLOBAL_DISPATCHER.as_mut()
        };

        if let Some(dispatcher) = maybe_dispatcher {
            f(*dispatcher)
        }
    }
}

impl Drop for GlobalDispatcherHandle {
    fn drop(&mut self) {
        if let Some(restore_state) = self.restore_state {
            // SAFETY: restore state received from the corresponding call to `critical_sectino::acquire`
            unsafe { critical_section::release(restore_state) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_get() {
        let shared_dispatcher = SharedTestDispatcher::new();

        GlobalDispatcher::init_boxed(shared_dispatcher.clone()).unwrap();

        shared_dispatcher.assert_bytes(&[]);

        let bytes = [1, 2, 3];

        GlobalDispatcher::global_dispatcher().get(|dispatcher| dispatcher.write(&bytes));

        shared_dispatcher.assert_bytes(&bytes);

        assert!(GlobalDispatcher::init_boxed(shared_dispatcher).is_err());
    }
}
