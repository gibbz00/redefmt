use crate::*;

pub enum NormalizedValue {
    Primitive(Primitive),
    Slice(Slice),
    Array(Array),
    Tuple(Tuple),
    Map(Map),
    Set(Set),
    Type(FormatDocumentId),
}
