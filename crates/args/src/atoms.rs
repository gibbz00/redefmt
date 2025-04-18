// integer is a decimal integer that may contain leading zeroes and must fit into an usize
pub struct Integer(usize);

pub enum ArgumentIdentifier {
    Index(Integer),
    // IMPROVEMENT: borrowed?
    Identifier(String),
}
