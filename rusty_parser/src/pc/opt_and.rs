use crate::pc::{Parser, Token, Tokenizer, Undo};
use crate::{binary_parser_declaration, ParseError, ParserErrorTrait};

// The left side is optional, the right is not.
// If the right is missing, the left is reverted.

binary_parser_declaration!(pub struct OptAndPC);

impl<I: Tokenizer + 'static, L, R> Parser<I> for OptAndPC<L, R>
where
    L: Parser<I, Output = Token>,
    R: Parser<I>,
{
    type Output = (Option<Token>, R::Output);
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        let opt_leading = self.left.parse_opt(tokenizer)?;
        match self.right.parse(tokenizer) {
            Ok(value) => Ok((opt_leading, value)),
            Err(err) => {
                if err.is_incomplete() {
                    opt_leading.undo(tokenizer);
                }
                Err(err)
            }
        }
    }
}
