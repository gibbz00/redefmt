#[derive(Debug, Clone, Copy, PartialEq, derive_more::AsRef)]
pub struct Stamp(u64);

impl Stamp {
    pub fn new(inner: u64) -> Self {
        Self(inner)
    }
}
