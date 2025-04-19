use alloc::string::String;

use crate::*;

pub enum ArgumentIdentifier {
    Index(Integer),
    // IMPROVEMENT: borrowed?
    Identifier(String),
}
