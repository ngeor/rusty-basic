use crate::{InputTrait, Parser, ParserErrorTrait};

pub struct OrDefaultParser<P> {
    parser: P,
}
impl<P> OrDefaultParser<P> {
    pub(crate) fn new(parser: P) -> Self {
        Self { parser }
    }
}
impl<I, C, P> Parser<I, C> for OrDefaultParser<P>
where
    I: InputTrait,
    P: Parser<I, C>,
    P::Output: Default,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(value) => Ok(value),
            Err(err) if err.is_soft() => Ok(P::Output::default()),
            Err(err) => Err(err),
        }
    }

    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
