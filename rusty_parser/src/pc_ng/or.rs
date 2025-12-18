use crate::pc_ng::*;

pub struct Or<I, O, E> {
    parsers: Vec<Box<dyn Parser<Input = I, Output = O, Error = E>>>,
}

impl<I, O, E> Or<I, O, E> {
    pub fn new(parsers: Vec<Box<dyn Parser<Input = I, Output = O, Error = E>>>) -> Self {
        Self { parsers }
    }
}

impl<I, O, E> Parser for Or<I, O, E> {
    type Input = I;
    type Output = O;
    type Error = E;

    fn parse(&self, mut input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        for parser in &self.parsers {
            match parser.parse(input) {
                ParseResult::Ok(i, result) => return ParseResult::Ok(i, result),
                ParseResult::None(remaining) | ParseResult::Expected(remaining, _) => {
                    input = remaining;
                }
                ParseResult::Err(i, err) => return ParseResult::Err(i, err),
            }
        }
        ParseResult::None(input)
    }
}
