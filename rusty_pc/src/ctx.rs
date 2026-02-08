use std::marker::PhantomData;

use crate::{InputTrait, Parser, ParserErrorTrait};

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
    I: InputTrait,
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
    I: InputTrait,
    C: Clone,
    E: ParserErrorTrait,
{
    type Output = C;
    type Error = E;

    fn parse(&mut self, _input: &mut I) -> Result<Self::Output, Self::Error> {
        match &self.0 {
            Some(ctx) => Ok(ctx.clone()),
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

pub struct NoContextParser<P, C1, C2> {
    parser: P,
    _marker: PhantomData<(C1, C2)>,
}

impl<P, C1, C2> NoContextParser<P, C1, C2> {
    pub(crate) fn new(parser: P) -> Self {
        Self {
            parser,
            _marker: PhantomData,
        }
    }
}

impl<I, C1, C2, P> Parser<I, C2> for NoContextParser<P, C1, C2>
where
    P: Parser<I, C1>,
    I: InputTrait,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        self.parser.parse(input)
    }
}

impl<C1, C2, P> SetContext<C2> for NoContextParser<P, C1, C2> {
    fn set_context(&mut self, _ctx: C2) {}
}

/// Based on the boolean context, parses using the left or the right parser.
pub struct IifParser<L, R> {
    left: L,
    right: R,
    context: bool,
}

impl<L, R> IifParser<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self {
            left,
            right,
            context: false,
        }
    }
}

impl<L, R, I> Parser<I, bool> for IifParser<L, R>
where
    I: InputTrait,
    L: Parser<I>,
    R: Parser<I, Output = L::Output, Error = L::Error>,
{
    type Output = L::Output;
    type Error = L::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        if self.context {
            self.left.parse(input)
        } else {
            self.right.parse(input)
        }
    }
}

impl<L, R> SetContext<bool> for IifParser<L, R> {
    fn set_context(&mut self, ctx: bool) {
        self.context = ctx;
    }
}
