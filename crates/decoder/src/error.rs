use std::string::FromUtf8Error;

use encode_unicode::error::Utf8Error;
use redefmt_common::{
    codec::frame::TypeHint,
    identifiers::{CrateId, PrintStatementId},
};
use redefmt_db::{DbClientError, crate_table::CrateName, state_dir::StateDirError};

#[derive(Debug, thiserror::Error)]
pub enum RedefmtDecoderError {
    #[error("state directory resolution error")]
    StateDir(#[from] StateDirError),
    #[error("database failure")]
    Db(#[from] DbClientError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("unknown bits present in header '{0:?}'")]
    UnknownHeader(u8),
    #[error("no crate with ID '{0}' registered")]
    UnknownCrate(CrateId),
    #[error("no print statement with ID '{0}' registered for '{1}'")]
    UnknownPrintStatement(PrintStatementId, CrateName<'static>),
    #[error("type hint '{0}' not recognized")]
    UnknownTypeHint(u8),
    #[error("invalid bytes received for the given type hint '{0}', bytes: '{1:?}'")]
    InvalidValueBytes(TypeHint, Vec<u8>),
    #[error("content length '{0}' does not fit host usize and will overflow")]
    LengthOverflow(u64),
    #[error("invalid UTF-8 bytes received for string type hint")]
    InvalidStringBytes(#[from] FromUtf8Error),
    #[error("invalid character byte length received, max should be 4")]
    InvalidCharLength(u8),
    #[error("invalid UTF-8 character bytes")]
    InvalidUtf8Char(#[from] Utf8Error),
}
