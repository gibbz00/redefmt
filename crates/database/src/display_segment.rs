use redefmt_args::FormatOptions;

use crate::*;

pub enum NormalizedSegment {
    String(String),
    Arg(NormalizedValue, FormatOptions<'static>),
}
