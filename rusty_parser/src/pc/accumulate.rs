use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{binary_parser_declaration, ParseError};

binary_parser_declaration!(pub struct AccumulateParser);

impl<I: Tokenizer + 'static, L, R> Parser<I> for AccumulateParser<L, R>
where
    L: Parser<I>,
    R: Parser<I, Output = L::Output>,
{
    type Output = Vec<L::Output>;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.left.parse(tokenizer).flat_map(|first| {
            let mut result: Vec<L::Output> = vec![];
            result.push(first);
            loop {
                match self.right.parse(tokenizer) {
                    ParseResult::Ok(next) => {
                        result.push(next);
                    }
                    ParseResult::None | ParseResult::Expected(_) => {
                        break;
                    }
                    ParseResult::Err(err) => {
                        return ParseResult::Err(err);
                    }
                }
            }

            ParseResult::Ok(result)
        })
    }
}
