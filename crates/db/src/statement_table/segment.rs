use std::borrow::Cow;

use redefmt_args::FormatOptions;

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Segment<'a> {
    Str(Cow<'a, str>),
    #[serde(borrow)]
    Arg(FormatOptions<'a>),
}
