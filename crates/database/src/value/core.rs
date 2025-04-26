use crate::*;

pub enum Value {
    Primitive,
    Slice(Slice),
    Array(Array),
    Tuple(Tuple),
    Map(Map),
    Set(Set),
    Type(Type),
}
