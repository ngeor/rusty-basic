use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::{NonOptParser, Parser, Tokenizer};
use crate::parser_declaration;

struct StaticErrorMapper {
    target: QError,
}

impl StaticErrorMapper {
    pub fn ensure_complete(target: QError) -> Self {
        debug_assert!(!target.is_incomplete());
        Self { target }
    }

    pub fn ensure_incomplete(target: QError) -> Self {
        debug_assert!(target.is_incomplete());
        Self { target }
    }

    fn map_err(&self, err: QError) -> QError {
        if err.is_incomplete() {
            self.target.clone()
        } else {
            err
        }
    }
}

pub struct MapIncompleteErrParser<P> {
    parser: P,
    err: StaticErrorMapper,
}

impl<P> MapIncompleteErrParser<P> {
    pub fn new(parser: P, err: QError) -> Self {
        Self {
            parser,
            err: StaticErrorMapper::ensure_incomplete(err),
        }
    }
}

impl<P> Parser for MapIncompleteErrParser<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser
            .parse(tokenizer)
            .map_err(|err| self.err.map_err(err))
    }
}

pub struct OrFailParser<P> {
    parser: P,
    err: StaticErrorMapper,
}

impl<P> OrFailParser<P> {
    pub fn new(parser: P, err: QError) -> Self {
        Self {
            parser,
            err: StaticErrorMapper::ensure_complete(err),
        }
    }
}

impl<P> Parser for OrFailParser<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser
            .parse(tokenizer)
            .map_err(|err| self.err.map_err(err))
    }
}

impl<P> NonOptParser for OrFailParser<P> where P: Parser {}

parser_declaration!(struct NoIncompleteParser);

impl<P> Parser for NoIncompleteParser<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser
            .parse(tokenizer)
            .map_err(ParserErrorTrait::no_incomplete)
    }
}

impl<P> NonOptParser for NoIncompleteParser<P> where P: Parser {}
