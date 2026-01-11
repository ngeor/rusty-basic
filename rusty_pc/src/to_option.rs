use crate::{ParseResult, Parser, SetContext};

pub trait ToOption<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn to_option(self) -> ToOptionParser<Self> {
        ToOptionParser::new(self)
    }
}
impl<I, C, P> ToOption<I, C> for P where P: Parser<I, C> {}

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
    P: Parser<I, C>,
{
    type Output = Option<P::Output>;
    type Error = P::Error;
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok((input, value)) => Ok((input, Some(value))),
            Err((false, input, _)) => Ok((input, None)),
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
