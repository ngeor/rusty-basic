use crate::{InputTrait, Parser, SetContext};

pub struct FlattenParser<P> {
    parser: P,
}
impl<P> FlattenParser<P> {
    pub(crate) fn new(parser: P) -> Self {
        Self { parser }
    }
}
impl<I, C, P> Parser<I, C> for FlattenParser<P>
where
    I: InputTrait,
    P: Parser<I, C>,
    P::Output: Parser<I, C, Error = P::Error>,
{
    type Output = <P::Output as Parser<I, C>>::Output;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(mut new_parser) => new_parser.parse(input),
            Err(err) => Err(err),
        }
    }
}
impl<C, P> SetContext<C> for FlattenParser<P>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
