use redefmt_args::FormatOptions;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum Segment<'a> {
    String(String),
    #[serde(borrow)]
    Arg(FormatOptions<'a>),
}
