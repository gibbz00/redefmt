use std::env::VarError;

use proc_macro::TokenStream;
use proc_macro2::Span;
use redefmt_db::{DbClientError, StateDirError, crate_table::CrateNameError};

#[derive(Debug, thiserror::Error)]
pub enum RedefmtMacroError {
    #[error("state directory resolution error")]
    StateDir(#[from] StateDirError),
    #[error("internal database failure")]
    Db(#[from] DbClientError),
    #[error("failed to retrieve crate name from '$CARGO_PKG_NAME'")]
    CrateNameEnv(#[from] VarError),
    #[error("invalid crate name")]
    CrateName(#[from] CrateNameError),
}

impl RedefmtMacroError {
    pub fn as_compiler_error(&self, span: Span) -> TokenStream {
        syn::Error::new(span, self).into_compile_error().into()
    }
}
