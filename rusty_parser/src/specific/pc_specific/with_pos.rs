use crate::parser_declaration;
use crate::pc::{ParseResult, ParseResultTrait, Parser};
use rusty_common::{AtPos, HasPos, Positioned};

parser_declaration!(pub struct WithPosMapper);

impl<I: HasPos, P> Parser<I> for WithPosMapper<P>
where
    P: Parser<I>,
{
    type Output = Positioned<P::Output>;
    type Error = P::Error;
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        let pos = tokenizer.pos();
        self.parser.parse(tokenizer).map_ok(|x| x.at_pos(pos))
    }
}
