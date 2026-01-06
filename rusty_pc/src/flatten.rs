use crate::{ParseResult, Parser};

pub trait Flatten<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn flatten(self) -> FlattenParser<Self>
    where
        Self::Output: Parser<I, C, Error = Self::Error>,
    {
        FlattenParser::new(self)
    }
}
impl<I, C, P> Flatten<I, C> for P where P: Parser<I, C> {}

pub struct FlattenParser<P> {
    parser: P,
}
impl<P> FlattenParser<P> {
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}
impl<I, C, P> Parser<I, C> for FlattenParser<P>
where
    P: Parser<I, C>,
    P::Output: Parser<I, C, Error = P::Error>,
{
    type Output = <P::Output as Parser<I, C>>::Output;
    type Error = P::Error;
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok((i, new_parser)) => new_parser.parse(i),
            Err(err) => Err(err),
        }
    }
}
impl<C, P> crate::SetContext<C> for FlattenParser<P>
where
    P: crate::SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
