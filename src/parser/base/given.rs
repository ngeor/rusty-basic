//
// Different way to build parsers
//

use crate::common::QError;
use crate::parser::base::and_pc::AndDemandRef;
use crate::parser::base::parsers::{HasOutput, NonOptParser, Parser};
use crate::parser::base::tokenizers::Tokenizer;

pub struct Builder<G>
where
    G: Parser,
{
    given: G,
}

pub fn given<G>(given: G) -> Builder<G>
where
    G: Parser,
{
    Builder { given }
}

impl<G> Builder<G>
where
    G: Parser,
{
    pub fn then<T>(self, then: T) -> Builder2<G, T>
    where
        T: NonOptParser,
    {
        Builder2 {
            given: self.given,
            then,
        }
    }
}

pub struct Builder2<G, T>
where
    G: Parser,
    T: NonOptParser,
{
    given: G,
    then: T,
}

impl<G, T> HasOutput for Builder2<G, T>
where
    G: Parser,
    T: NonOptParser,
{
    type Output = (G::Output, T::Output);
}

impl<G, T> Parser for Builder2<G, T>
where
    G: Parser,
    T: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        AndDemandRef::new(&self.given, &self.then).parse(tokenizer)
    }
}
