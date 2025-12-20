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
        R: Parser<I, Output = Self::Output> + 'static;
}

impl<I, P> Either<I> for P
where
    I: Clone,
    P: Parser<I> + 'static,
{
    fn or<R>(self, other: R) -> impl Parser<I, Output = Self::Output>
    where
        R: Parser<I, Output = Self::Output> + 'static,
    {
        OrParser::new(vec![Box::new(self), Box::new(other)])
    }
}
