use crate::pc::{Parser, Tokenizer, Undo};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct PeekParser);

impl<I: Tokenizer + 'static, P> Parser<I> for PeekParser<P>
where
    P: Parser<I>,
    P::Output: Undo,
{
    type Output = ();

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(|item| {
            item.undo(tokenizer);
        })
    }
}
