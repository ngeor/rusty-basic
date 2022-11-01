use crate::parser_declaration;
use crate::pc::{Parser, Tokenizer, Undo};
use rusty_common::*;

parser_declaration!(pub struct PeekParser);

impl<P> Parser for PeekParser<P>
where
    P: Parser,
    P::Output: Undo,
{
    type Output = ();

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(|item| {
            item.undo(tokenizer);
        })
    }
}