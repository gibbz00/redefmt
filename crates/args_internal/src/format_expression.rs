use crate::*;

/// Validated and optimized format string together with its required arguments
#[derive(Debug, PartialEq)]
pub struct FormatExpression<'a, E> {
    format_string: FormatString<'a>,
    provided_args: ProvidedArgs<'a, E>,
}

impl<'a, E> FormatExpression<'a, E> {
    pub fn new<C: ArgCapturer<E>>(
        mut format_string: FormatString<'a>,
        mut provided_args: ProvidedArgs<'a, E>,
        arg_capturer: Option<&C>,
    ) -> Result<Self, ResolveArgsError>
    where
        E: PartialEq,
    {
        InternalArgumentResolver::resolve(&mut format_string, &mut provided_args, arg_capturer)?;
        Ok(Self { format_string, provided_args })
    }

    pub fn dissolve(self) -> (DeferredFormatString<'a>, ProvidedArgs<'a, E>) {
        let deferred_format_expression = DeferredFormatString { format_string: self.format_string };

        (deferred_format_expression, self.provided_args)
    }
}

#[cfg(feature = "syn")]
impl syn::parse::Parse for FormatExpression<'static, syn::Expr> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let comma = syn::Token![,];
        let format_string = input.parse()?;

        let provided_args = match input.peek(comma) {
            true => {
                let _ = input.parse::<syn::Token![,]>()?;
                input.parse()?
            }
            false => {
                if !input.is_empty() {
                    return Err(syn::Error::new(
                        input.span(),
                        "provided args tokens not separated by comma",
                    ));
                }

                Default::default()
            }
        };

        Self::new(format_string, provided_args, Some(&SynArgCapturer))
            .map_err(|error| syn::Error::new(input.span(), error))
    }
}
