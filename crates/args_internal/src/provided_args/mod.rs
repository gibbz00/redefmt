mod resolver;
pub(crate) use resolver::{ArgumentResolver, ResolveArgsError};

mod expression {
    use crate::*;

    pub trait Expression {
        fn from_identifier(any_identifier: AnyIdentifier<'_>) -> Self;

        fn unmove_expression(&mut self);
    }

    impl Expression for syn::Expr {
        fn from_identifier(any_identifier: AnyIdentifier<'_>) -> Self {
            from_identifier_impl(any_identifier.into())
        }

        fn unmove_expression(&mut self) {
            if let syn::Expr::Path(path) = self
                && let Some(ident) = path.path.get_ident().cloned()
            {
                *self = from_identifier_impl(ident);
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
pub(crate) use expression::Expression;
