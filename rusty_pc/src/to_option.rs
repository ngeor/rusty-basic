use crate::{InputTrait, Parser, ParserErrorTrait, SetContext};

pub trait ToOption<I: InputTrait, C>: Parser<I, C>
where
    Self: Sized,
{
    fn to_option(self) -> ToOptionParser<Self> {
        ToOptionParser::new(self)
    }
}
impl<I, C, P> ToOption<I, C> for P
where
    I: InputTrait,
    P: Parser<I, C>,
{
}

pub struct ToOptionParser<P> {
    parser: P,
}
impl<P> ToOptionParser<P> {
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}
impl<I, C, P> Parser<I, C> for ToOptionParser<P>
where
    I: InputTrait,
    P: Parser<I, C>,
{
    type Output = Option<P::Output>;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(value) => Ok(Some(value)),
            Err(err) if !err.is_fatal() => Ok(None),
            Err(err) => Err(err),
        }
    }
}
impl<C, P> SetContext<C> for ToOptionParser<P>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
