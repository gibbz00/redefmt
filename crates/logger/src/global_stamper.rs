use core::sync::atomic::{AtomicU8, Ordering};

use crate::*;

static mut GLOBAL_STAMPER: Option<&'static dyn Stamper> = None;

static GLOBAL_STAMPER_STATE: AtomicU8 = AtomicU8::new(UNINITIALIZED);

const UNINITIALIZED: u8 = 0;
const INITIALIZING: u8 = 1;
const INITIALIZED: u8 = 2;

pub struct GlobalStamper;

#[derive(Debug)]
pub enum GlobalStamperError {
    AlreadyInitialized,
}

impl GlobalStamper {
    pub fn init(stamper: &'static dyn Stamper) -> Result<(), GlobalStamperError> {
        GLOBAL_STAMPER_STATE
            .compare_exchange(UNINITIALIZED, INITIALIZING, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| GlobalStamperError::AlreadyInitialized)?;

        // SAFETY: static not yet initialized, check and initializing flag is
        // atomic and sequentially consistent
        unsafe { GLOBAL_STAMPER = Some(stamper) };

        GLOBAL_STAMPER_STATE.store(INITIALIZED, Ordering::SeqCst);

        Ok(())
    }

    pub fn stamper() -> Option<&'static dyn Stamper> {
        match GLOBAL_STAMPER_STATE.load(Ordering::Acquire) == INITIALIZED {
            true => {
                // SAFETY: no mutation of static done after INITIALIZED state
                unsafe {
                    #[allow(static_mut_refs)]
                    GLOBAL_STAMPER
                }
            }
            false => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_stamp() {
        static TEST_STAMPER: TestStamper = TestStamper::new();

        assert!(GlobalStamper::stamper().is_none());

        GlobalStamper::init(&TEST_STAMPER).unwrap();

        assert!(GlobalStamper::stamper().is_some());

        let stamper = GlobalStamper::stamper().unwrap();

        let first_stamp = stamper.stamp();
        let second_stamp = stamper.stamp();

        assert_ne!(first_stamp, second_stamp);

        assert!(GlobalStamper::init(&TEST_STAMPER).is_err());
    }
}
