use crate::*;

#[derive(Debug, PartialEq)]
pub enum Parameter {
    Index(Integer),
    Identifier(Identifier),
}

impl Parse for Parameter {
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        match str.starts_with(|ch: char| ch.is_ascii_digit()) {
            true => Integer::parse(offset, str).map(Parameter::Index),
            false => Identifier::parse(offset, str).map(Parameter::Identifier),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_index() {
        let str = "01";

        let expected_parameter = {
            let integer = Integer::parse(0, str).unwrap();
            Parameter::Index(integer)
        };

        let actual_parameter = Parameter::parse(0, str).unwrap();

        assert_eq!(expected_parameter, actual_parameter);
    }

    #[test]
    fn parse_identifier() {
        let str = "x";

        let expected_parameter = {
            let identifier = Identifier::parse(0, str).unwrap();
            Parameter::Identifier(identifier)
        };

        let actual_parameter = Parameter::parse(0, str).unwrap();

        assert_eq!(expected_parameter, actual_parameter);
    }
}
