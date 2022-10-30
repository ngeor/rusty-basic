use crate::parser_declaration;
use crate::pc::{Parser, Tokenizer};
use rusty_common::{AtLocation, Locatable, QError};

parser_declaration!(pub struct WithPosMapper);

impl<P> Parser for WithPosMapper<P>
where
    P: Parser,
{
    type Output = Locatable<P::Output>;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let pos = tokenizer.position();
        self.parser.parse(tokenizer).map(|x| x.at(pos))
    }
}
