use std::marker::PhantomData;

use crate::{InputTrait, Parser, ParserErrorTrait};

/// A parser that always succeeds, providing the value returned by the given function.
pub fn supplier<I, C, F, O, E>(f: F) -> impl Parser<I, C, Output = O, Error = E>
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

    fn set_context(&mut self, _ctx: C) {}
}

/// A parser that always fails, providing the value returned by the given function.
pub fn err_supplier<I, C, F, O, E>(f: F) -> impl Parser<I, C, Output = O, Error = E>
where
    I: InputTrait,
    F: Fn() -> E,
    E: ParserErrorTrait,
{
    ErrSupplierParser(f, PhantomData)
}

struct ErrSupplierParser<F, O>(F, PhantomData<O>);

impl<I, C, F, O, E> Parser<I, C> for ErrSupplierParser<F, O>
where
    I: InputTrait,
    F: Fn() -> E,
    E: ParserErrorTrait,
{
    type Output = O;
    type Error = E;

    fn parse(&mut self, _input: &mut I) -> Result<O, E> {
        Err((self.0)())
    }

    fn set_context(&mut self, _ctx: C) {}
}
