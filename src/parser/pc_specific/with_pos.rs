use crate::common::{AtLocation, Locatable, QError};
use crate::parser::pc::*;

pub struct WithPosMapper<P>(P);

impl<P> ParserBase for WithPosMapper<P>
where
    P: ParserBase,
{
    type Output = Locatable<P::Output>;
}

impl<P> OptParser for WithPosMapper<P>
where
    P: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let pos = tokenizer.position();
        self.0
            .parse(tokenizer)
            .map(|opt_x| opt_x.map(|x| x.at(pos)))
    }
}

impl<P> NonOptParser for WithPosMapper<P>
where
    P: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let pos = tokenizer.position();
        self.0.parse(tokenizer).map(|x| x.at(pos))
    }
}

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
