use crate::{InputTrait, Parser};

/// Initializes the context of the underlying parser to the given value
/// before parsing starts.
pub struct InitContextParser<P, CInner> {
    /// The underlying parser.
    parser: P,
    /// The value to set the context to.
    value: CInner,
}

impl<P, CInner> InitContextParser<P, CInner> {
    /// Creates a new instance.
    pub(crate) fn new(parser: P, value: CInner) -> Self {
        Self { parser, value }
    }
}

impl<I, COuter, CInner, P> Parser<I, COuter> for InitContextParser<P, CInner>
where
    I: InputTrait,
    P: Parser<I, CInner>,
    CInner: Clone,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        self.parser.set_context(self.value.clone());
        self.parser.parse(input)
    }

    fn set_context(&mut self, _ctx: COuter) {}
}
