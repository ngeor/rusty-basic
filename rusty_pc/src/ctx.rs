use std::marker::PhantomData;

use crate::{ParseResult, Parser};

/// Access the context as a parser.
pub fn ctx_parser<I, C, E>() -> impl Parser<I, C, Output = C, Error = E>
where
    C: Clone,
    E: Default,
{
    CtxParser::new(None)
}

struct CtxParser<C, E>(Option<C>, PhantomData<E>);

impl<C, E> CtxParser<C, E> {
    pub fn new(ctx: Option<C>) -> Self {
        Self(ctx, PhantomData)
    }
}

impl<I, C, E> Parser<I, C> for CtxParser<C, E>
where
    C: Clone,
    E: Default,
{
    type Output = C;
    type Error = E;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match &self.0 {
            Some(ctx) => Ok((input, ctx.clone())),
            None => Err((true, input, E::default())),
        }
    }

    fn set_context(&mut self, ctx: C) {
        // This is the actual point where the context gets stored.
        // All other parser combinators are supposed to propagate
        // it with the `in_context` method, recreating themselves
        // and invoking `in_context` on the parsers they decorate.
        self.0 = Some(ctx);
    }
}

pub trait NoContext<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn no_context<C2>(self) -> impl Parser<I, C2, Output = Self::Output, Error = Self::Error> {
        NoContextParser::new(self)
    }
}
impl<I, C, P> NoContext<I, C> for P where P: Parser<I, C> {}

struct NoContextParser<P, C1, C2> {
    parser: P,
    _marker: PhantomData<(C1, C2)>,
}
impl<P, C1, C2> NoContextParser<P, C1, C2> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _marker: PhantomData,
        }
    }
}
impl<I, C1, C2, P> Parser<I, C2> for NoContextParser<P, C1, C2>
where
    P: Parser<I, C1>,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.parser.parse(input)
    }
    fn set_context(&mut self, _ctx: C2) {}
}
