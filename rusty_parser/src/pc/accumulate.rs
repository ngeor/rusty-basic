use crate::pc::{Parser, Tokenizer};
use crate::{binary_parser_declaration, ParseError};

binary_parser_declaration!(pub struct AccumulateParser);

impl<L, R> Parser for AccumulateParser<L, R>
where
    L: Parser,
    R: Parser<Output = L::Output>,
{
    type Output = Vec<L::Output>;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        let first = self.left.parse(tokenizer)?;
        let mut result: Vec<L::Output> = vec![];
        result.push(first);
        while let Some(next) = self.right.parse_opt(tokenizer)? {
            result.push(next);
        }
        Ok(result)
    }
}
