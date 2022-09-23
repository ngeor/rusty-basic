//
// Different way to build parsers
//

use crate::common::QError;
use crate::parser::base::and_pc::AndDemandRef;
use crate::parser::base::parsers::{HasOutput, NonOptParser, Parser};
use crate::parser::base::tokenizers::Tokenizer;

pub struct Builder<G> {
    given: G,
}

pub fn given<G>(given: G) -> Builder<G> {
    Builder { given }
}

impl<G> Builder<G>
where
    G: Parser,
{
    pub fn then<T>(self, then: T) -> Builder2<G, T> {
        Builder2 {
            given: self.given,
            then,
        }
    }
}

pub struct Builder2<G, T> {
    given: G,
    then: T,
}

impl<G, T> HasOutput for Builder2<G, T>
where
    G: HasOutput,
    T: HasOutput,
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

impl<G, T> NonOptParser for Builder2<G, T>
where
    G: NonOptParser,
    T: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        AndDemandRef::new(&self.given, &self.then).parse_non_opt(tokenizer)
    }
}
