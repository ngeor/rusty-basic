use crate::pc::{Parser, Tokenizer};
use crate::{parser_declaration, ParseError};
use rusty_common::{AtPos, Positioned};

parser_declaration!(pub struct WithPosMapper);

impl<P> Parser for WithPosMapper<P>
where
    P: Parser,
{
    type Output = Positioned<P::Output>;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        let pos = tokenizer.position();
        self.parser.parse(tokenizer).map(|x| x.at_pos(pos))
    }
}
