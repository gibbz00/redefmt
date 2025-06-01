use crate::*;

pub trait ArgCapturer {
    type Expression;

    fn transform_identifier(&self, any_identifier: AnyIdentifier<'_>) -> Self::Expression;

    /// Unmove expressions
    ///
    /// Rust's `format_args!` compiles:
    ///
    /// ```rust
    /// let x = "x".to_string();
    /// print!("{} {x}", x, x = x);
    /// drop(x);
    /// ```
    ///
    /// ...because the print statement i transformed into: `print!("{} {x}", &x, x = &x);`
    fn unmove_expression(&self, expression: &mut Self::Expression);
}

#[cfg(feature = "syn")]
mod syn_impl {
    use super::*;

    pub struct SynArgCapturer;

    impl ArgCapturer for SynArgCapturer {
        type Expression = syn::Expr;

        fn transform_identifier(&self, any_identifier: AnyIdentifier<'_>) -> syn::Expr {
            from_identifier_impl(any_identifier.into())
        }

        fn unmove_expression(&self, expression: &mut syn::Expr) {
            if let syn::Expr::Path(path) = expression
                && let Some(ident) = path.path.get_ident().cloned()
            {
                *expression = from_identifier_impl(ident);
            }
        }
    }

    fn from_identifier_impl(ident: syn::Ident) -> syn::Expr {
        syn::Expr::Reference(syn::ExprReference {
            attrs: Default::default(),
            and_token: Default::default(),
            mutability: None,
            expr: alloc::boxed::Box::new(syn::Expr::Path(syn::ExprPath {
                attrs: Default::default(),
                qself: None,
                path: syn::Path::from(ident),
            })),
        })
    }
}
#[cfg(feature = "syn")]
pub use syn_impl::SynArgCapturer;
