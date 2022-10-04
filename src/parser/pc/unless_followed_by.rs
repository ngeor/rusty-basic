use crate::binary_parser_declaration;
use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::{Parser, Tokenizer, Undo};
binary_parser_declaration!(pub struct UnlessFollowedByParser);

impl<L, R> Parser for UnlessFollowedByParser<L, R>
where
    L: Parser,
    R: Parser,
    L::Output: Undo,
    R::Output: Undo,
{
    type Output = L::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let first = self.left.parse(tokenizer)?;

        match self.right.parse(tokenizer) {
            Ok(second) => {
                second.undo(tokenizer);
                first.undo(tokenizer);
                Err(QError::Incomplete)
            }
            Err(err) if err.is_incomplete() => Ok(first),
            Err(err) => Err(err),
        }
    }
}
