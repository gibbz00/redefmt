use crate::*;

pub trait Format {
    fn fmt(&self, f: &mut Formatter);
}

impl<T: WriteValue> Format for T {
    fn fmt(&self, f: &mut Formatter) {
        f.write(self);
    }
}

pub struct Formatter<'a> {
    dispatcher: &'a mut dyn Dispatcher,
}

impl<'a> Formatter<'a> {
    pub fn new(dispatcher: &'a mut dyn Dispatcher) -> Self {
        Self { dispatcher }
    }

    // Monomorphization should be ok since `WriteValue` is sealed
    pub fn write(&mut self, value: impl WriteValue) {
        value.write_value(self.dispatcher);
    }
}
