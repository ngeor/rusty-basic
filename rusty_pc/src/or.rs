use crate::{ParseResult, Parser, SetContext};

pub struct OrParser<I, C, O, E> {
    parsers: Vec<Box<dyn Parser<I, C, Output = O, Error = E>>>,
}

impl<I, C, O, E> OrParser<I, C, O, E> {
    pub fn new(parsers: Vec<Box<dyn Parser<I, C, Output = O, Error = E>>>) -> Self {
        Self { parsers }
    }
}

impl<I, C, O, E> Parser<I, C> for OrParser<I, C, O, E>
where
    C: Clone,
{
    type Output = O;
    type Error = E;

    fn parse(&self, mut input: I) -> ParseResult<I, O, Self::Error> {
        for i in 0..self.parsers.len() - 1 {
            match self.parsers[i].parse(input) {
                Ok(x) => return Ok(x),
                Err((false, i, _)) => {
                    input = i;
                    continue;
                }
                Err(err) => return Err(err),
            }
        }

        self.parsers.last().unwrap().parse(input)
    }
}

pub struct OrParserNoBox<L, R> {
    left: L,
    right: R,
}
impl<L, R> OrParserNoBox<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}
impl<C, L, R> SetContext<C> for OrParserNoBox<L, R>
where
    C: Clone,
    L: SetContext<C>,
    R: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.left.set_context(ctx.clone());
        self.right.set_context(ctx);
    }
}

pub trait Or<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn or<R>(self, other: R) -> OrParserNoBox<Self, R>
    where
        R: Parser<I, C, Output = Self::Output, Error = Self::Error>,
    {
        OrParserNoBox::new(self, other)
    }
}

impl<I, C, P> Or<I, C> for P where P: Parser<I, C> {}

impl<I, C, L, R> Parser<I, C> for OrParserNoBox<L, R>
where
    L: Parser<I, C>,
    R: Parser<I, C, Output = L::Output, Error = L::Error>,
{
    type Output = L::Output;
    type Error = L::Error;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.left.parse(input) {
            Ok(x) => Ok(x),
            Err((false, input, _)) => self.right.parse(input),
            Err(err) => Err(err),
        }
    }
}
