use crate::pc::{Parser, Tokenizer};
use crate::{parser_declaration, ParseError};
use rusty_common::{AtPos, Positioned};

parser_declaration!(pub struct WithPosMapper);

impl<I: Tokenizer + 'static, P> Parser<I> for WithPosMapper<P>
where
    P: Parser<I>,
{
    type Output = Positioned<P::Output>;
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        let pos = tokenizer.position();
        self.parser.parse(tokenizer).map(|x| x.at_pos(pos))
    }
}
