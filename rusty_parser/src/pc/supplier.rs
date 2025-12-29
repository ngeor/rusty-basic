use std::marker::PhantomData;

use crate::pc::{ParseResult, Parser};

pub fn supplier<I, F, O, E>(f: F) -> impl Parser<I, Output = O, Error = E>
where
    F: Fn() -> O,
{
    SupplierParser(f, PhantomData)
}

struct SupplierParser<F, E>(F, PhantomData<E>);

impl<I, F, O, E> Parser<I> for SupplierParser<F, E>
where
    F: Fn() -> O,
{
    type Output = O;
    type Error = E;

    fn parse(&self, input: I) -> ParseResult<I, O, E> {
        Ok((input, (self.0)()))
    }
}
