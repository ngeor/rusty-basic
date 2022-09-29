//
// Map
//

// TODO make QError generic param too after figuring out <T> vs associated type

use crate::common::QError;
use crate::parser::pc::{NonOptParser, OptParser, ParserBase, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct FnMapper<mapper: F>);

impl<P, F, U> ParserBase for FnMapper<P, F>
where
    P: ParserBase,
    F: Fn(P::Output) -> U,
{
    type Output = U;
}

impl<P, F, U> OptParser for FnMapper<P, F>
where
    P: OptParser,
    F: Fn(P::Output) -> U,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.parser
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(&self.mapper))
    }
}

impl<P, F, U> NonOptParser for FnMapper<P, F>
where
    P: NonOptParser,
    F: Fn(P::Output) -> U,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse_non_opt(tokenizer).map(&self.mapper)
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

impl<P, L, R> OptParser for KeepLeftMapper<P>
where
    P: OptParser<Output = (L, R)>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.parser
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(|(l, _)| l))
    }
}

impl<P, L, R> NonOptParser for KeepLeftMapper<P>
where
    P: NonOptParser<Output = (L, R)>,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse_non_opt(tokenizer).map(|(l, _)| l)
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

impl<P, L, M, R> OptParser for KeepMiddleMapper<P>
where
    P: OptParser<Output = ((L, M), R)>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.parser
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(|((_, m), _)| m))
    }
}

impl<P, L, M, R> NonOptParser for KeepMiddleMapper<P>
where
    P: NonOptParser<Output = ((L, M), R)>,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse_non_opt(tokenizer).map(|((_, m), _)| m)
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

impl<P, L, R> OptParser for KeepRightMapper<P>
where
    P: OptParser<Output = (L, R)>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.parser
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(|(_, r)| r))
    }
}

impl<P, L, R> NonOptParser for KeepRightMapper<P>
where
    P: NonOptParser<Output = (L, R)>,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse_non_opt(tokenizer).map(|(_, r)| r)
    }
}
