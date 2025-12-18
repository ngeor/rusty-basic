use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError, ParserErrorTrait};

struct StaticErrorMapper {
    target: ParseError,
}

impl StaticErrorMapper {
    pub fn ensure_complete(target: ParseError) -> Self {
        debug_assert!(!target.is_incomplete());
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

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(value),
            ParseResult::None => {
                ParseResult::Err(self.static_error_mapper.map_err(ParseError::Incomplete))
            }
            ParseResult::Err(err) => ParseResult::Err(self.static_error_mapper.map_err(err)),
        }
    }
}

parser_declaration!(pub struct NoIncompleteParser);

impl<I: Tokenizer + 'static, P> Parser<I> for NoIncompleteParser<P>
where
    P: Parser<I>,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(value),
            ParseResult::None => {
                ParseResult::Err(ParserErrorTrait::no_incomplete(ParseError::Incomplete))
            }
            ParseResult::Err(err) => ParseResult::Err(ParserErrorTrait::no_incomplete(err)),
        }
    }
}

pub struct WithExpectedMessage<P, F>(P, F);

impl<P, F> WithExpectedMessage<P, F> {
    pub fn new(parser: P, f: F) -> Self {
        Self(parser, f)
    }
}

pub trait MessageProvider {
    fn to_str(&self) -> String;
}

impl MessageProvider for &str {
    fn to_str(&self) -> String {
        self.to_string()
    }
}

impl MessageProvider for String {
    fn to_str(&self) -> String {
        self.clone()
    }
}

impl<F> MessageProvider for F
where
    F: Fn() -> String,
{
    fn to_str(&self) -> String {
        (self)()
    }
}

impl<I: Tokenizer + 'static, P, F> Parser<I> for WithExpectedMessage<P, F>
where
    P: Parser<I>,
    F: MessageProvider,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.0.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(value),
            ParseResult::None => ParseResult::Err(ParseError::Expected(self.1.to_str())),
            ParseResult::Err(err) if err.is_incomplete() => {
                ParseResult::Err(ParseError::Expected(self.1.to_str()))
            }
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
