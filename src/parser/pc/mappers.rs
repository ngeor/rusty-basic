//
// Map
//

// TODO make QError generic param too after figuring out <T> vs associated type

use crate::common::QError;
use crate::parser::pc::{Parser, ParserBase, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct FnMapper<mapper: F>);

impl<P, F, U> ParserBase for FnMapper<P, F>
where
    P: ParserBase,
    F: Fn(P::Output) -> U,
{
    type Output = U;
}

impl<P, F, U> Parser for FnMapper<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> U,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(&self.mapper)
    }
}

//
// Keep Left
//

parser_declaration!(struct KeepLeftMapper);

impl<P, L, R> ParserBase for KeepLeftMapper<P>
where
    P: ParserBase<Output = (L, R)>,
{
    type Output = L;
}

impl<P, L, R> Parser for KeepLeftMapper<P>
where
    P: Parser<Output = (L, R)>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(|(l, _)| l)
    }
}

//
// Keep Middle
//

parser_declaration!(struct KeepMiddleMapper);

impl<P, L, M, R> ParserBase for KeepMiddleMapper<P>
where
    P: ParserBase<Output = ((L, M), R)>,
{
    type Output = M;
}

impl<P, L, M, R> Parser for KeepMiddleMapper<P>
where
    P: Parser<Output = ((L, M), R)>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(|((_, m), _)| m)
    }
}

//
// Keep Right
//

parser_declaration!(struct KeepRightMapper);

impl<P, L, R> ParserBase for KeepRightMapper<P>
where
    P: ParserBase<Output = (L, R)>,
{
    type Output = R;
}

impl<P, L, R> Parser for KeepRightMapper<P>
where
    P: Parser<Output = (L, R)>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(|(_, r)| r)
    }
}
