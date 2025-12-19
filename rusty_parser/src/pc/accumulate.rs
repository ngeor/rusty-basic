use crate::pc::{ParseResult, ParseResultTrait, Parser};
use crate::{binary_parser_declaration, ParseError};

binary_parser_declaration!(pub struct AccumulateParser);

impl<I, L, R> Parser<I> for AccumulateParser<L, R>
where
    L: Parser<I>,
    R: Parser<I, Output = L::Output>,
{
    type Output = Vec<L::Output>;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, ParseError> {
        self.left.parse(input).flat_map(|input, first| {
            let mut result: Vec<L::Output> = vec![];
            result.push(first);
            let mut input = input;
            loop {
                match self.right.parse(input) {
                    Ok((i, next)) => {
                        result.push(next);
                        input = i;
                    }
                    Err((false, i, _)) => {
                        input = i;
                        break;
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            Ok((input, result))
        })
    }
}
