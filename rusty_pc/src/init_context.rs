use crate::{ParseResult, Parser, SetContext};

pub trait InitContext<I, CInner>: Parser<I, CInner> + SetContext<CInner> + Sized {
    /// Creates a parser that will initialize the context of the underlying parser
    /// to the given value before parsing starts.
    fn init_context(self, value: CInner) -> InitContextParser<Self, CInner>
    where
        CInner: Clone,
    {
        InitContextParser::new(self, value)
    }
}

impl<I, CInner, P> InitContext<I, CInner> for P where P: Parser<I, CInner> + SetContext<CInner> {}

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
    pub fn new(parser: P, value: CInner) -> Self {
        Self { parser, value }
    }
}

impl<I, COuter, CInner, P> Parser<I, COuter> for InitContextParser<P, CInner>
where
    P: Parser<I, CInner> + SetContext<CInner>,
    CInner: Clone,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.parser.set_context(self.value.clone());
        self.parser.parse(input)
    }
}
