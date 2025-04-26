use crate::*;

// TODO: seal?
pub trait WritePrimitive {
    fn write_primitive(&self, dispatcher: &dyn Dispatcher);
}

impl WritePrimitive for &[u8] {
    fn write_primitive(&self, dispatcher: &dyn Dispatcher) {
        self.len().write_primitive(dispatcher);
        dispatcher.write(self);
    }
}

impl WritePrimitive for &str {
    fn write_primitive(&self, dispatcher: &dyn Dispatcher) {
        self.len().write_primitive(dispatcher);
        dispatcher.write(self.as_bytes());
    }
}

impl<const N: usize> WritePrimitive for [u8; N] {
    fn write_primitive(&self, dispatcher: &dyn Dispatcher) {
        // No need to send length, should have been registered in write!/log! statement
        dispatcher.write(self);
    }
}

impl WritePrimitive for bool {
    fn write_primitive(&self, dispatcher: &dyn Dispatcher) {
        (*self as u8).write_primitive(dispatcher);
    }
}

macro_rules! num_impl {
    ($($type:ty,)*) => {
        $(
            impl WritePrimitive for $type {
                fn write_primitive(&self, dispatcher: &dyn Dispatcher) {
                    dispatcher.write(&self.to_be_bytes());
                }
            }
        )*
    };
}

num_impl!(i8, i16, i32, i64, isize, u8, u16, u32, u64, u128, usize, f32, f64,);
