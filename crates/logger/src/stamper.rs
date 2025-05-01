use redefmt_common::codec::Stamp;

pub trait Stamper {
    fn stamp(&self) -> Stamp;
}
