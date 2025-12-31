use crate::*;

pub struct OrParser<I, C, O, E> {
    parsers: Vec<Box<dyn Parser<I, C, Output = O, Error = E>>>,
}

impl<I, C, O, E> OrParser<I, C, O, E> {
    pub fn new(parsers: Vec<Box<dyn Parser<I, C, Output = O, Error = E>>>) -> Self {
        Self { parsers }
    }
}

impl<I, C, O, E> Parser<I, C> for OrParser<I, C, O, E> {
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

pub trait Or<I, C>: Parser<I, C>
where
    Self: Sized + 'static,
{
    fn or<R>(self, other: R) -> OrParser<I, C, Self::Output, Self::Error>
    where
        R: Parser<I, C, Output = Self::Output, Error = Self::Error> + 'static,
    {
        OrParser::new(vec![Box::new(self), Box::new(other)])
    }
}

impl<I, C, P> Or<I, C> for P where P: Parser<I, C> + 'static {}
