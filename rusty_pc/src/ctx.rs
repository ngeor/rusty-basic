use std::marker::PhantomData;

use crate::{InputTrait, Parser, ParserErrorTrait};

/// Access the context as a parser.
pub fn ctx_parser<I, C, E>() -> impl Parser<I, C, Output = C, Error = E>
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

    fn set_context(&mut self, ctx: &C) {
        // This is the actual point where the context gets stored.
        // All other parser combinators are supposed to propagate
        // it with the `in_context` method, recreating themselves
        // and invoking `in_context` on the parsers they decorate.
        self.0 = Some(ctx.clone());
    }
}
