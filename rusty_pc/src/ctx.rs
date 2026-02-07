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

/// Stores the context and delegates parsing to the
/// parser that is returned by a parser factory
/// which uses the context to create the parser.
pub struct FnCtxParser<C, F, C2> {
    /// The context
    context: Option<C>,

    /// The parser factory
    parser_factory: F,

    /// Marker data storing the context type of the created parser
    /// (different than the context type of this parser)
    _marker: PhantomData<C2>,
}

impl<C, F, C2> FnCtxParser<C, F, C2> {
    pub fn new<I, P>(parser_factory: F) -> Self
    where
        F: Fn(&C) -> P,
        P: Parser<I, C2>,
        I: InputTrait,
    {
        Self {
            context: None,
            parser_factory,
            _marker: PhantomData,
        }
    }
}

impl<C, F, C2> SetContext<C> for FnCtxParser<C, F, C2> {
    fn set_context(&mut self, ctx: C) {
        self.context = Some(ctx);
    }
}

impl<I, C, F, P, C2> Parser<I, C> for FnCtxParser<C, F, C2>
where
    I: InputTrait,
    F: Fn(&C) -> P,
    P: Parser<I, C2>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        let mut parser = (self.parser_factory)(&self.context.as_ref().unwrap());
        parser.parse(input)
    }
}
