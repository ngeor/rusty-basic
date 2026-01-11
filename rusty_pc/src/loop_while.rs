use crate::{ParseResult, Parser, SetContext, default_parse_error};

pub trait LoopWhile<I, C>: Parser<I, C>
where
    Self: Sized,
    Self::Error: Default,
{
    fn loop_while<F>(
        self,
        predicate: F,
    ) -> impl Parser<I, C, Output = Vec<Self::Output>, Error = Self::Error>
    where
        F: Fn(&Self::Output) -> bool,
    {
        LoopWhileParser::new(self, predicate)
    }
}
impl<I, C, P> LoopWhile<I, C> for P
where
    P: Parser<I, C>,
    P::Error: Default,
{
}

struct LoopWhileParser<P, F> {
    parser: P,
    predicate: F,
}
impl<P, F> LoopWhileParser<P, F> {
    pub fn new(parser: P, predicate: F) -> Self {
        Self { parser, predicate }
    }
}
impl<I, C, P, F> Parser<I, C> for LoopWhileParser<P, F>
where
    P: Parser<I, C>,
    P::Error: Default,
    F: Fn(&P::Output) -> bool,
{
    type Output = Vec<P::Output>;
    type Error = P::Error;
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        let mut remaining = tokenizer;
        while keep_going {
            match self.parser.parse(remaining) {
                Ok((tokenizer, item)) => {
                    keep_going = (self.predicate)(&item);
                    result.push(item);
                    remaining = tokenizer;
                }
                Err((false, i, _)) => {
                    remaining = i;
                    keep_going = false;
                }
                Err(err) => return Err(err),
            }
        }
        if result.is_empty() {
            default_parse_error(remaining)
        } else {
            Ok((remaining, result))
        }
    }
}
impl<C, P, F> SetContext<C> for LoopWhileParser<P, F>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
