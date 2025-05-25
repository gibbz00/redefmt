use crate::*;

#[derive(Debug, PartialEq)]
pub struct FormatExpression<'a> {
    format_string: FormatString<'a>,
    provided_args: ProvidedArgs<'a>,
}

impl FormatExpression<'_> {
    pub fn format_string(&self) -> &FormatString {
        &self.format_string
    }

    pub fn provided_args(&self) -> &ProvidedArgs {
        &self.provided_args
    }
}

impl<'a> FormatExpression<'a> {
    pub fn new(
        mut format_string: FormatString<'a>,
        mut provided_args: ProvidedArgs<'a>,
    ) -> Result<Self, ResolveArgsError> {
        ArgumentResolver::resolve(&mut format_string, &mut provided_args)?;
        Ok(Self { format_string, provided_args })
    }
}

impl<'a> From<FormatExpression<'a>> for MappedFormatExpression<'a> {
    fn from(expression: FormatExpression<'a>) -> Self {
        Self {
            format_string: expression.format_string,
            provided_args: expression.provided_args.into(),
        }
    }
}

impl syn::parse::Parse for FormatExpression<'static> {
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

        Self::new(format_string, provided_args).map_err(|error| syn::Error::new(input.span(), error))
    }
}
