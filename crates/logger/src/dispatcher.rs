// TODO: make unsafe?
pub trait Dispatcher {
    fn acquire(&self);

    fn write(&self, bytes: &[u8]);

    /// Should flush also flush the the internal buffer
    fn release(&self);
}
