use crate::pc::*;
use crate::ParseError;

pub struct OrParser<I, O> {
    parsers: Vec<Box<dyn Parser<I, Output = O>>>,
}

impl<I, O> OrParser<I, O> {
    pub fn new(parsers: Vec<Box<dyn Parser<I, Output = O>>>) -> Self {
        Self { parsers }
    }
}

impl<I, O> Parser<I> for OrParser<I, O> {
    type Output = O;
    fn parse(&self, mut input: I) -> ParseResult<I, O, ParseError> {
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

pub trait Either<I: Clone>: Parser<I> {
    fn or<R>(self, other: R) -> impl Parser<I, Output = Self::Output>
    where
        R: Parser<I, Output = Self::Output>;
}

impl<I, P> Either<I> for P
where
    I: Clone,
    P: Parser<I>,
{
    fn or<R>(self, other: R) -> impl Parser<I, Output = Self::Output>
    where
        R: Parser<I, Output = Self::Output>,
    {
        EitherParser::new(self, other)
    }
}

pub struct EitherParser<L, R> {
    left: L,
    right: R,
}

impl<L, R> EitherParser<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

impl<I, L, R> Parser<I> for EitherParser<L, R>
where
    L: Parser<I>,
    R: Parser<I, Output = L::Output>,
{
    type Output = L::Output;

    fn parse(&self, input: I) -> ParseResult<I, L::Output, ParseError> {
        match self.left.parse(input) {
            Ok(x) => Ok(x),
            Err((false, input, _)) => self.right.parse(input),
            Err(err) => Err(err),
        }
    }
}
