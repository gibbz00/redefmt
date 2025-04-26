use crate::*;

pub struct GlobalRegistry;

impl GlobalRegistry {
    pub fn initialize(distatcher: &'static dyn Dispatcher, stamper: Option<&'static dyn Stamper>) {
        todo!()
    }

    pub(crate) fn dispatcher() -> &'static dyn Dispatcher {
        // methods for setting the global dispatcher
        todo!()
    }

    pub(crate) fn stamper() -> Option<&'static dyn Stamper> {
        todo!()
    }
}
