//
// Map
//

use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct FnMapper<mapper: F>);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for FnMapper<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(&self.mapper)
    }
}

//
// Keep Left
//

parser_declaration!(pub struct KeepLeftMapper);

impl<I: Tokenizer + 'static, P, L, R> Parser<I> for KeepLeftMapper<P>
where
    P: Parser<I, Output = (L, R)>,
{
    type Output = L;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(|(l, _)| l)
    }
}

//
// Keep Right
//

parser_declaration!(pub struct KeepRightMapper);

impl<I: Tokenizer + 'static, P, L, R> Parser<I> for KeepRightMapper<P>
where
    P: Parser<I, Output = (L, R)>,
{
    type Output = R;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(|(_, r)| r)
    }
}
