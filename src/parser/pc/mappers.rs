//
// Map
//

use crate::common::QError;
use crate::parser::pc::{NonOptParser, Parser, ParserOnce, Tokenizer};
use crate::parser_declaration;

parser_declaration!(pub struct FnMapper<mapper: F>);

// TODO: question, can a macro reduce the repetition of the impl traits

impl<P, F, U> Parser for FnMapper<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(&self.mapper)
    }
}

impl<P, F, U> ParserOnce for FnMapper<P, F>
where
    P: ParserOnce,
    F: FnOnce(P::Output) -> U,
{
    type Output = U;
    fn parse(self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(self.mapper)
    }
}

impl<P, F, U> NonOptParser for FnMapper<P, F>
where
    P: NonOptParser,
    F: Fn(P::Output) -> U,
{
}

//
// Keep Left
//

parser_declaration!(pub struct KeepLeftMapper);

impl<P, L, R> Parser for KeepLeftMapper<P>
where
    P: Parser<Output = (L, R)>,
{
    type Output = L;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(|(l, _)| l)
    }
}

impl<P, L, R> NonOptParser for KeepLeftMapper<P> where P: NonOptParser<Output = (L, R)> {}

//
// Keep Right
//

parser_declaration!(pub struct KeepRightMapper);

impl<P, L, R> Parser for KeepRightMapper<P>
where
    P: Parser<Output = (L, R)>,
{
    type Output = R;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(|(_, r)| r)
    }
}

impl<P, L, R> NonOptParser for KeepRightMapper<P> where P: NonOptParser<Output = (L, R)> {}
