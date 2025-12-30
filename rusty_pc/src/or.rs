use crate::*;

pub struct OrParser<I, O, E> {
    parsers: Vec<Box<dyn Parser<I, Output = O, Error = E>>>,
    allow_incomplete: bool,
}

impl<I, O, E> OrParser<I, O, E> {
    pub fn new(parsers: Vec<Box<dyn Parser<I, Output = O, Error = E>>>) -> Self {
        Self {
            parsers,
            allow_incomplete: true,
        }
    }
}

impl<I, O, E> Parser<I> for OrParser<I, O, E> {
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

        self.parsers
            .last()
            .unwrap()
            .parse(input)
            .flat_map_err(|fatal, i, err| Err((fatal || !self.allow_incomplete, i, err)))
    }

    fn no_incomplete(self) -> impl parsers::Parser<I, Output = O, Error = E> {
        Self {
            allow_incomplete: false,
            ..self
        }
    }
}

pub trait Or<I>: Parser<I>
where
    Self: Sized + 'static,
{
    fn or<R>(self, other: R) -> OrParser<I, Self::Output, Self::Error>
    where
        R: Parser<I, Output = Self::Output, Error = Self::Error> + 'static,
    {
        OrParser::new(vec![Box::new(self), Box::new(other)])
    }
}

impl<I, P> Or<I> for P where P: Parser<I> + 'static {}
