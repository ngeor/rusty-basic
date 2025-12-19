//
// Map
//

use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

// Map using the given function.

parser_declaration!(pub struct Map<mapper: F>);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for Map<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.parser.parse(tokenizer).map(&self.mapper)
    }
}

// Map Ok and None using a trait.
// Both Ok and None get mapped to an Ok value.
// TODO Therefore this is a NonOptParser.

pub struct MapOkNone<I: Tokenizer + 'static, O, U> {
    parser: Box<dyn Parser<I, Output = O>>,
    mapper: Box<dyn MapOkNoneTrait<O, U>>,
}

impl<I: Tokenizer + 'static, O, U> MapOkNone<I, O, U> {
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

impl<I: Tokenizer + 'static, O, U> Parser<I> for MapOkNone<I, O, U> {
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

pub struct MapToOption;

impl<T> MapOkNoneTrait<T, Option<T>> for MapToOption {
    fn map_ok(&self, value: T) -> Option<T> {
        Some(value)
    }

    fn map_none(&self) -> Option<T> {
        None
    }
}

pub struct MapToDefault;

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
