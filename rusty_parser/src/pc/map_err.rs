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

impl<I: Tokenizer + 'static, P> Parser<I> for MapIncompleteErrParser<P>
where
    P: Parser<I>,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
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

impl<I: Tokenizer + 'static, P> Parser<I> for OrFailParser<P>
where
    P: Parser<I>,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        self.parser
            .parse(tokenizer)
            .map_err(|err| self.static_error_mapper.map_err(err))
    }
}

impl<I: Tokenizer + 'static, P> NonOptParser<I> for OrFailParser<P> where P: Parser<I> {}

parser_declaration!(pub struct NoIncompleteParser);

impl<I: Tokenizer + 'static, P> Parser<I> for NoIncompleteParser<P>
where
    P: Parser<I>,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        self.parser
            .parse(tokenizer)
            .map_err(ParserErrorTrait::no_incomplete)
    }
}

impl<I: Tokenizer + 'static, P> NonOptParser<I> for NoIncompleteParser<P> where P: Parser<I> {}
