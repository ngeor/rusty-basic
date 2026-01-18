use std::marker::PhantomData;

use crate::{InputTrait, Parser, ParserErrorTrait, SetContext};

pub fn supplier<I, C, F, O, E>(f: F) -> impl Parser<I, C, Output = O, Error = E> + SetContext<C>
where
    I: InputTrait,
    F: Fn() -> O,
    E: ParserErrorTrait,
{
    SupplierParser(f, PhantomData)
}

struct SupplierParser<F, E>(F, PhantomData<E>);

impl<I, C, F, O, E> Parser<I, C> for SupplierParser<F, E>
where
    I: InputTrait,
    F: Fn() -> O,
    E: ParserErrorTrait,
{
    type Output = O;
    type Error = E;

    fn parse(&mut self, _input: &mut I) -> Result<O, E> {
        Ok((self.0)())
    }
}

impl<C, P, F> SetContext<C> for SupplierParser<P, F> {
    fn set_context(&mut self, _ctx: C) {}
}
