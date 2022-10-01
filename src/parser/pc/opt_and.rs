use crate::binary_parser_declaration;
use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::{Parser, Token, Tokenizer, Undo};

// The left side is optional, the right is not.
// If the right is missing, the left is reverted.

binary_parser_declaration!(struct OptAndPC);

impl<L, R> Parser for OptAndPC<L, R>
where
    L: Parser<Output = Token>,
    R: Parser,
{
    type Output = (Option<Token>, R::Output);
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let opt_leading = self.0.parse_opt(tokenizer)?;
        match self.1.parse(tokenizer) {
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
