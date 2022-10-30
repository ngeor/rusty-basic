use crate::binary_parser_declaration;
use crate::pc::{NonOptParser, Parser, Token, Tokenizer, Undo};
use rusty_common::{ParserErrorTrait, QError};

// The left side is optional, the right is not.
// If the right is missing, the left is reverted.

binary_parser_declaration!(pub struct OptAndPC);

impl<L, R> Parser for OptAndPC<L, R>
where
    L: Parser<Output = Token>,
    R: Parser,
{
    type Output = (Option<Token>, R::Output);
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
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

impl<L, R> NonOptParser for OptAndPC<L, R>
where
    L: Parser<Output = Token>,
    R: NonOptParser,
{
}
