use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::ParseError;

pub trait MapOkNone<I: Tokenizer + 'static>: Parser<I> {
    /// Map the result of this parser for successful and incomplete results.
    /// The given mapper implements [MapOkNoneTrait] which takes care of the mapping.
    fn map_ok_none<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized + 'static,
        F: MapOkNoneTrait<Self::Output, U> + 'static;

    fn to_option(self) -> impl Parser<I, Output = Option<Self::Output>>
    where
        Self: Sized + 'static,
    {
        self.map_ok_none(MapToOption)
    }

    fn or_default(self) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized + 'static,
        Self::Output: Default,
    {
        self.map_ok_none(MapToDefault)
    }
}

impl<I, P> MapOkNone<I> for P
where
    I: Tokenizer + 'static,
    P: Parser<I>,
{
    fn map_ok_none<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized + 'static,
        F: MapOkNoneTrait<Self::Output, U> + 'static,
    {
        MapOkNoneParser::new(self, mapper)
    }
}

// Map Ok and None using a trait.
// Both Ok and None get mapped to an Ok value.
// TODO Therefore this is a NonOptParser.

struct MapOkNoneParser<I: Tokenizer + 'static, O, U> {
    parser: Box<dyn Parser<I, Output = O>>,
    mapper: Box<dyn MapOkNoneTrait<O, U>>,
}

impl<I: Tokenizer + 'static, O, U> MapOkNoneParser<I, O, U> {
    pub fn new(
        parser: impl Parser<I, Output = O> + 'static,
        mapper: impl MapOkNoneTrait<O, U> + 'static,
    ) -> Self {
        Self {
            parser: Box::new(parser),
            mapper: Box::new(mapper),
        }
    }
}

impl<I: Tokenizer + 'static, O, U> Parser<I> for MapOkNoneParser<I, O, U> {
    type Output = U;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok((self.mapper).map_ok(value)),
            ParseResult::None | ParseResult::Expected(_) => {
                ParseResult::Ok((self.mapper).map_none())
            }
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}

pub trait MapOkNoneTrait<T, U> {
    fn map_ok(&self, value: T) -> U;

    fn map_none(&self) -> U;
}

struct MapToOption;

impl<T> MapOkNoneTrait<T, Option<T>> for MapToOption {
    fn map_ok(&self, value: T) -> Option<T> {
        Some(value)
    }

    fn map_none(&self) -> Option<T> {
        None
    }
}

struct MapToDefault;

impl<T> MapOkNoneTrait<T, T> for MapToDefault
where
    T: Default,
{
    fn map_ok(&self, value: T) -> T {
        value
    }

    fn map_none(&self) -> T {
        T::default()
    }
}
