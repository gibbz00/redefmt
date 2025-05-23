mod identifier;
pub(crate) use identifier::Identifier;

mod any_identifier;
pub(crate) use any_identifier::AnyIdentifier;

mod error;
pub(crate) use error::IdentifierParseError;

mod utils;
