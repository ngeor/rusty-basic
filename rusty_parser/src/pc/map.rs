//
// Map
//

use std::marker::PhantomData;

use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

// Map using the given function.

parser_declaration!(pub struct MapPC<mapper: F>);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for MapPC<P, F>
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

pub struct MapOkNoneTraitPC<P, F, O> {
    parser: P,
    mapper: F,
    _marker: PhantomData<O>,
}

impl<P, F, O> MapOkNoneTraitPC<P, F, O> {
    pub fn new(parser: P, mapper: F) -> Self {
        Self {
            parser,
            mapper,
            _marker: PhantomData,
        }
    }
}

impl<I: Tokenizer + 'static, P, F, O> Parser<I> for MapOkNoneTraitPC<P, F, O>
where
    P: Parser<I>,
    F: MapOk<P::Output, O>,
{
    type Output = O;

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

pub trait MapOk<T, U> {
    fn map_ok(&self, value: T) -> U;

    fn map_none(&self) -> U;
}

pub struct MapToOption;

impl<T> MapOk<T, Option<T>> for MapToOption {
    fn map_ok(&self, value: T) -> Option<T> {
        Some(value)
    }

    fn map_none(&self) -> Option<T> {
        None
    }
}

pub struct MapToDefault;

impl<T> MapOk<T, T> for MapToDefault
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
