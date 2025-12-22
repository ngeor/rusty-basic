use crate::{
    pc::{ParseResult, Parser},
    ParseError,
};

pub fn supplier<I, F, O>(f: F) -> impl Parser<I, Output = O>
where
    F: Fn() -> O,
{
    SupplierParser(f)
}

struct SupplierParser<F>(F);

impl<I, F, O> Parser<I> for SupplierParser<F>
where
    F: Fn() -> O,
{
    type Output = O;

    fn parse(&self, input: I) -> ParseResult<I, O, ParseError> {
        Ok((input, (self.0)()))
    }
}
