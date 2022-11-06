use crate::pc::{NonOptParser, Parser, Tokenizer};
use crate::{parser_declaration, ParseError, ParserErrorTrait};

struct StaticErrorMapper {
    target: ParseError,
}

impl StaticErrorMapper {
    pub fn ensure_complete(target: ParseError) -> Self {
        debug_assert!(!target.is_incomplete());
        Self { target }
    }

    pub fn ensure_incomplete(target: ParseError) -> Self {
        debug_assert!(target.is_incomplete());
        Self { target }
    }

    fn map_err(&self, err: ParseError) -> ParseError {
        if err.is_incomplete() {
            self.target.clone()
        } else {
            err
        }
    }
}

pub struct MapIncompleteErrParser<P> {
    parser: P,
    static_error_mapper: StaticErrorMapper,
}

impl<P> MapIncompleteErrParser<P> {
    pub fn new(parser: P, err: ParseError) -> Self {
        Self {
            parser,
            static_error_mapper: StaticErrorMapper::ensure_incomplete(err),
        }
    }
}

impl<P> Parser for MapIncompleteErrParser<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        self.parser
            .parse(tokenizer)
            .map_err(|err| self.static_error_mapper.map_err(err))
    }
}

pub struct OrFailParser<P> {
    parser: P,
    static_error_mapper: StaticErrorMapper,
}

impl<P> OrFailParser<P> {
    pub fn new(parser: P, err: ParseError) -> Self {
        Self {
            parser,
            static_error_mapper: StaticErrorMapper::ensure_complete(err),
        }
    }
}

impl<P> Parser for OrFailParser<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        self.parser
            .parse(tokenizer)
            .map_err(|err| self.static_error_mapper.map_err(err))
    }
}

impl<P> NonOptParser for OrFailParser<P> where P: Parser {}

parser_declaration!(pub struct NoIncompleteParser);

impl<P> Parser for NoIncompleteParser<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        self.parser
            .parse(tokenizer)
            .map_err(ParserErrorTrait::no_incomplete)
    }
}

impl<P> NonOptParser for NoIncompleteParser<P> where P: Parser {}
