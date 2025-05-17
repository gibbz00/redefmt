//! # redefmt-macros

use proc_macro::TokenStream;

#[proc_macro_derive(Format, attributes(redefmt))]
pub fn derive_format(token_stream: TokenStream) -> TokenStream {
    todo!()
}
