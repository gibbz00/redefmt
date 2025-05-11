use redefmt_common::codec::Stamp;

pub trait Stamper {
    fn stamp(&self) -> Stamp;
}

#[cfg(test)]
mod test_stamper {
    use core::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    pub struct TestStamper(AtomicU64);

    impl TestStamper {
        pub const fn new() -> Self {
            Self(AtomicU64::new(0))
        }
    }

    impl Stamper for TestStamper {
        fn stamp(&self) -> Stamp {
            let inner = self.0.fetch_add(1, Ordering::Relaxed);
            Stamp::new(inner)
        }
    }
}
#[cfg(test)]
pub(crate) use test_stamper::TestStamper;
