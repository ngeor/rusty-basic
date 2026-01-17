use std::marker::PhantomData;

use crate::{ParseResult, Parser, ParserErrorTrait};

/// Indicates the support of context and allows for setting the context value.
pub trait SetContext<C> {
    /// Stores the given context.
    /// A parser that can store context needs to override this method.
    /// Parsers that delegate to other parsers should implement this method
    /// by propagating the context to the delegate.
    fn set_context(&mut self, ctx: C);
}

/// Access the context as a parser.
pub fn ctx_parser<I, C, E>() -> impl Parser<I, C, Output = C, Error = E> + SetContext<C>
where
    C: Clone,
    E: ParserErrorTrait,
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
    E: ParserErrorTrait,
{
    type Output = C;
    type Error = E;

    fn parse(&mut self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match &self.0 {
            Some(ctx) => Ok((input, ctx.clone())),
            None => panic!("context was not set"),
        }
    }
}

impl<C, E> SetContext<C> for CtxParser<C, E> {
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
    fn no_context<C2>(self) -> NoContextParser<Self, C, C2> {
        NoContextParser::new(self)
    }
}
impl<I, C, P> NoContext<I, C> for P where P: Parser<I, C> {}

pub struct NoContextParser<P, C1, C2> {
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
    fn parse(&mut self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.parser.parse(input)
    }
}

impl<C1, C2, P> SetContext<C2> for NoContextParser<P, C1, C2> {
    fn set_context(&mut self, _ctx: C2) {}
}
