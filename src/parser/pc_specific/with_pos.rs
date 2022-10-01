use crate::common::{AtLocation, Locatable, QError};
use crate::parser::pc::*;

pub struct WithPosMapper<P>(P);

impl<P> ParserBase for WithPosMapper<P>
where
    P: ParserBase,
{
    type Output = Locatable<P::Output>;
}

impl<P> Parser for WithPosMapper<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let pos = tokenizer.position();
        self.0.parse(tokenizer).map(|x| x.at(pos))
    }
}

// TODO remove the traits from pc_specific too
pub trait WithPosTrait {
    fn with_pos(self) -> WithPosMapper<Self>
    where
        Self: Sized;
}

impl<S> WithPosTrait for S
where
    S: ParserBase,
{
    fn with_pos(self) -> WithPosMapper<Self> {
        WithPosMapper(self)
    }
}
