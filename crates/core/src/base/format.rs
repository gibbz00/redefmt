use crate::*;

pub trait Format {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result;
}

impl<T: Format> Format for &T {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        <T as Format>::fmt(self, f)
    }
}

pub struct Formatter<'a> {
    pub(crate) dispatcher: &'a mut dyn Dispatcher,
}

impl<'a> Formatter<'a> {
    pub fn new(dispatcher: &'a mut dyn Dispatcher) -> Self {
        Self { dispatcher }
    }

    pub fn write_statements<'r>(&'r mut self) -> StatementWriter<'r, 'a> {
        StatementWriter::init(self)
    }
}

#[cfg(feature = "deferred")]
impl<'a> Formatter<'a> {
    // Monomorphization should be ok here since `WriteValue` is sealed
    #[doc(hidden)]
    pub fn write_deferred(&mut self, value: impl WriteValue) {
        value.write_value(self.dispatcher);
    }

    #[doc(hidden)]
    pub fn write_raw_deferred(&mut self, value: impl WriteValue) {
        value.write_raw(self.dispatcher);
    }
}
