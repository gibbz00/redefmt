use crate::*;

#[derive(Default)]
pub struct ResolverConfig<C: ArgCapturer> {
    pub arg_capturer: Option<C>,
}
