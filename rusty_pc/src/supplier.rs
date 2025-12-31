use std::marker::PhantomData;

use crate::{ParseResult, Parser};

pub fn supplier<I, C, F, O, E>(f: F) -> impl Parser<I, C, Output = O, Error = E>
where
    F: Fn() -> O,
{
    SupplierParser(f, PhantomData)
}

struct SupplierParser<F, E>(F, PhantomData<E>);

impl<I, C, F, O, E> Parser<I, C> for SupplierParser<F, E>
where
    F: Fn() -> O,
{
    type Output = O;
    type Error = E;

    fn parse(&self, input: I) -> ParseResult<I, O, E> {
        Ok((input, (self.0)()))
    }
}
