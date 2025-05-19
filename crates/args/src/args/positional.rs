use syn::parse::Parse;

use crate::*;

pub struct PositionalArg(pub ArgValue);

impl Parse for PositionalArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse().map(Self)
    }
}
