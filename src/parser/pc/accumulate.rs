use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::{Parser, Tokenizer};

binary_parser_declaration!(struct AccumulateParser);

impl<L, R> Parser for AccumulateParser<L, R>
where
    L: Parser,
    R: Parser<Output = L::Output>,
{
    type Output = Vec<L::Output>;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let first = self.0.parse(tokenizer)?;
        let mut result: Vec<L::Output> = vec![];
        result.push(first);
        while let Some(next) = self.1.parse_opt(tokenizer)? {
            result.push(next);
        }
        Ok(result)
    }
}
