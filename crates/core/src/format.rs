use crate::*;

pub trait Format {
    fn fmt(&self, f: &mut Formatter);
}

impl<T: Format> Format for &T {
    fn fmt(&self, f: &mut Formatter) {
        <T as Format>::fmt(self, f);
    }
}

pub struct Formatter<'a> {
    dispatcher: &'a mut dyn Dispatcher,
}

impl<'a> Formatter<'a> {
    pub fn new(dispatcher: &'a mut dyn Dispatcher) -> Self {
        Self { dispatcher }
    }

    // Monomorphization should be ok here since `WriteValue` is sealed
    pub fn write(&mut self, value: impl WriteValue) {
        value.write_value(self.dispatcher);
    }

    #[doc(hidden)]
    pub fn write_raw(&mut self, value: impl WriteValue) {
        value.write_raw(self.dispatcher);
    }
}
