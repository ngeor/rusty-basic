use crate::{InputTrait, Parser, ParserErrorTrait, SetContext};

pub struct OrParser<I, C, O, E>
where
    E: ParserErrorTrait,
{
    parsers: Vec<Box<dyn Parser<I, C, Output = O, Error = E>>>,
}

impl<I, C, O, E> OrParser<I, C, O, E>
where
    E: ParserErrorTrait,
{
    pub fn new(parsers: Vec<Box<dyn Parser<I, C, Output = O, Error = E>>>) -> Self {
        Self { parsers }
    }
}

impl<I, C, O, E> Parser<I, C> for OrParser<I, C, O, E>
where
    I: InputTrait,
    C: Clone,
    E: ParserErrorTrait,
{
    type Output = O;
    type Error = E;

    fn parse(&mut self, input: &mut I) -> Result<O, Self::Error> {
        let original_position = input.get_position();
        for i in 0..self.parsers.len() - 1 {
            match self.parsers[i].parse(input) {
                Ok(x) => return Ok(x),
                Err(err) if err.is_soft() => {
                    input.set_position(original_position);
                    continue;
                }
                Err(err) => return Err(err),
            }
        }

        self.parsers.last_mut().unwrap().parse(input)
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

pub trait Or<I: InputTrait, C>: Parser<I, C>
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

impl<I, C, P> Or<I, C> for P
where
    I: InputTrait,
    P: Parser<I, C>,
{
}

impl<I, C, L, R> Parser<I, C> for OrParserNoBox<L, R>
where
    I: InputTrait,
    L: Parser<I, C>,
    R: Parser<I, C, Output = L::Output, Error = L::Error>,
{
    type Output = L::Output;
    type Error = L::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.left.parse(input) {
            Ok(x) => Ok(x),
            Err(err) if err.is_soft() => self.right.parse(input),
            Err(err) => Err(err),
        }
    }
}
