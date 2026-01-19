use crate::{InputTrait, Parser, ParserErrorTrait, SetContext};

pub trait OrDefault<I: InputTrait, C>: Parser<I, C>
where
    Self: Sized,
    Self::Output: Default,
{
    fn or_default(self) -> OrDefaultParser<Self> {
        OrDefaultParser::new(self)
    }
}
impl<I, C, P> OrDefault<I, C> for P
where
    I: InputTrait,
    P: Parser<I, C>,
    P::Output: Default,
{
}

pub struct OrDefaultParser<P> {
    parser: P,
}
impl<P> OrDefaultParser<P> {
    pub fn new(parser: P) -> Self {
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
}
impl<C, P> SetContext<C> for OrDefaultParser<P>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
