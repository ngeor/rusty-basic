use crate::common::{AtRowCol, Locatable, QError};
use crate::parser::pc::*;

pub struct WithPosMapper<P>(P);

impl<P> HasOutput for WithPosMapper<P>
where
    P: HasOutput,
{
    type Output = Locatable<P::Output>;
}

impl<P> Parser for WithPosMapper<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let pos = tokenizer.position();
        self.0
            .parse(tokenizer)
            .map(|opt_x| opt_x.map(|x| x.at_rc(pos.row, pos.col)))
    }
}

impl<P> NonOptParser for WithPosMapper<P>
where
    P: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let pos = tokenizer.position();
        self.0
            .parse_non_opt(tokenizer)
            .map(|x| x.at_rc(pos.row, pos.col))
    }
}

pub trait WithPosTrait {
    fn with_pos(self) -> WithPosMapper<Self>
    where
        Self: Sized;
}

impl<S> WithPosTrait for S
where
    S: HasOutput,
{
    fn with_pos(self) -> WithPosMapper<Self> {
        WithPosMapper(self)
    }
}
