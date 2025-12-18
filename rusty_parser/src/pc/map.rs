//
// Map
//

use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct MapPC<mapper: F>);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for MapPC<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(&self.mapper)
    }
}
