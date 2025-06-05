#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stamp(u64);

impl Stamp {
    pub fn new(inner: u64) -> Self {
        Self(inner)
    }
}

impl AsRef<u64> for Stamp {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}
