use crate::pc_ng::*;

pub struct Or<I, O, E> {
    parsers: Vec<Box<dyn Parser<Input = I, Output = O, Error = E>>>,
}

impl<I, O, E> Or<I, O, E> {
    pub fn new(parsers: Vec<Box<dyn Parser<Input = I, Output = O, Error = E>>>) -> Self {
        Self { parsers }
    }
}

impl<I, O, E> Parser for Or<I, O, E>
where
    I: Clone,
{
    type Input = I;
    type Output = O;
    type Error = E;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        for i in 0..self.parsers.len() - 1 {
            if let Ok(x) = self.parsers[i].parse(input.clone()) {
                return Ok(x);
            }
        }

        self.parsers.last().unwrap().parse(input)
    }
}
