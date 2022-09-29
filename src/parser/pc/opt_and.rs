use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::{OptParser, ParserBase, Token, Tokenizer};

// The left side is optional, the right is not.
// If the right is missing, the left is reverted.

binary_parser_declaration!(struct OptAndPC);

impl<L, R> ParserBase for OptAndPC<L, R>
where
    L: ParserBase<Output = Token>,
    R: ParserBase,
{
    type Output = (Option<Token>, R::Output);
}

impl<L, R> OptParser for OptAndPC<L, R>
where
    L: OptParser<Output = Token>,
    R: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_leading = self.0.parse(tokenizer)?;
        match self.1.parse(tokenizer)? {
            Some(value) => Ok(Some((opt_leading, value))),
            None => {
                if let Some(token) = opt_leading {
                    tokenizer.unread(token);
                }
                Ok(None)
            }
        }
    }
}
