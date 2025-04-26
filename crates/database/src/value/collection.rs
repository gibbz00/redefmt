use alloc::{boxed::Box, vec::Vec};

use crate::*;

// requires sending length over wire
pub struct Slice {
    value: Box<NormalizedValue>,
}

pub struct Array {
    value: Box<NormalizedValue>,
    length: usize,
}

pub struct Tuple {
    values: Vec<NormalizedValue>,
}

// requires sending length over wire
pub struct Map {
    key: Box<NormalizedValue>,
    value: Box<NormalizedValue>,
}

// requires sending length over wire
// (effectively a slice)
pub struct Set {
    value: Box<NormalizedValue>,
}
