use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::{NonOptParser, OptParser, ParserBase, Tokenizer};
//
// The left side can be followed by an optional right.
//
binary_parser_declaration!(struct AndOptPC);

impl<L, R> ParserBase for AndOptPC<L, R>
where
    L: ParserBase,
    R: ParserBase,
{
    type Output = (L::Output, Option<R::Output>);
}

impl<L, R> OptParser for AndOptPC<L, R>
where
    L: OptParser,
    R: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(left) => {
                let opt_right = self.1.parse(tokenizer)?;
                Ok(Some((left, opt_right)))
            }
            None => Ok(None),
        }
    }
}

impl<L, R> NonOptParser for AndOptPC<L, R>
where
    L: NonOptParser,
    R: OptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let left = self.0.parse_non_opt(tokenizer)?;
        let opt_right = self.1.parse(tokenizer)?;
        Ok((left, opt_right))
    }
}
