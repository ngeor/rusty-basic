//
// Map
//

use crate::pc::{NonOptParser, Parser, ParserOnce, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct FnMapper<mapper: F>);

// TODO: question, can a macro reduce the repetition of the impl traits

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for FnMapper<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(&self.mapper)
    }
}

impl<I: Tokenizer + 'static, P, F, U> ParserOnce<I> for FnMapper<P, F>
where
    P: ParserOnce<I>,
    F: FnOnce(P::Output) -> U,
{
    type Output = U;
    fn parse(self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(self.mapper)
    }
}

impl<I: Tokenizer + 'static, P, F, U> NonOptParser<I> for FnMapper<P, F>
where
    P: NonOptParser<I>,
    F: Fn(P::Output) -> U,
{
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
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(|(l, _)| l)
    }
}

impl<I: Tokenizer + 'static, P, L, R> NonOptParser<I> for KeepLeftMapper<P> where
    P: NonOptParser<I, Output = (L, R)>
{
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
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(|(_, r)| r)
    }
}

impl<I: Tokenizer + 'static, P, L, R> NonOptParser<I> for KeepRightMapper<P> where
    P: NonOptParser<I, Output = (L, R)>
{
}
