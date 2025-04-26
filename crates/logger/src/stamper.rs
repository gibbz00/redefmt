use core::num::NonZeroU64;

pub trait Stamper {
    fn stamp(&self) -> u64;
}
