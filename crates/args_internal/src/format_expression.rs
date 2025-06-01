use crate::*;

/// Resolved format string together with its provided arguments
#[derive(Debug, PartialEq)]
pub struct FormatExpression<'a, E> {
    format_string: FormatString<'a>,
    provided_args: ProvidedArgs<'a, E>,
}

impl<'a, E> FormatExpression<'a, E> {
    /// Resolves provided args with those in [`FormatString`] using a [`ResolverConfig`]
    ///
    /// Format string argument disambigaution is always performed, regardless of configuration:
    ///
    /// Unnamed positional arguments are disambiguated to indexed positional
    /// arguments. If the index points to a named argument, then the positional
    /// argument is replaced with the corresponding named argument.
    ///
    /// ```rust
    /// let before = format!("{1} {}", 1, x = 2);
    /// let after = format!("{x} {0}", 1, x = 2);
    /// assert_eq!(before, after);
    /// ```
    pub fn new<C: ArgCapturer<Expression = E>>(
        mut format_string: FormatString<'a>,
        mut provided_args: ProvidedArgs<'a, E>,
        resolver_config: &ProcessorConfig<C>,
    ) -> Result<Self, ProcessorError>
    where
        E: PartialEq,
    {
        FormatProcessor::resolve(&mut format_string, &mut provided_args, resolver_config)?;
        Ok(Self { format_string, provided_args })
    }

    pub fn dissolve(self) -> (ProcessedFormatString<'a>, ProvidedArgs<'a, E>) {
        let deferred_format_expression = ProcessedFormatString(self.format_string);

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

        let resolver_config = ProcessorConfig { arg_capturer: Some(SynArgCapturer), ..Default::default() };
        Self::new(format_string, provided_args, &resolver_config).map_err(|error| syn::Error::new(input.span(), error))
    }
}
