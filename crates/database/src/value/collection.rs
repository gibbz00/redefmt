use alloc::{boxed::Box, vec::Vec};

use crate::*;

// requires sending length over wire
pub struct Slice {
    value: Box<Value>,
}

pub struct Array {
    value: Box<Value>,
    length: usize,
}

pub struct Tuple {
    values: Vec<Value>,
}

// requires sending length over wire
pub struct Map {
    key: Box<Value>,
    value: Box<Value>,
}

// requires sending length over wire
// (effectively a slice)
pub struct Set {
    value: Box<Value>,
}
