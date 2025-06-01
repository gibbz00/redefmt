use crate::*;

#[derive(Debug, PartialEq)]
pub struct FormatExpression<'a> {
    pub processed_format_string: ProcessedFormatString<'a>,
    pub provided_args: ProvidedStaticArgs<'a, syn::Expr>,
}

impl syn::parse::Parse for FormatExpression<'static> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let comma = syn::Token![,];

        let format_string = input.parse()?;

        let mut provided_args = match input.peek(comma) {
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

        let resolver_config = StaticProcessorConfig { arg_capturer: Some(SynArgCapturer), ..Default::default() };

        let processed_format_string =
            FormatProcessor::process_static(format_string, &mut provided_args, &resolver_config)
                .map_err(|error| syn::Error::new(input.span(), error))?;

        Ok(Self { processed_format_string, provided_args })
    }
}
