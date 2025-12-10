use crate::pc_ng::*;

pub struct Many<P, S, A> {
    parser: P,
    seed: S,
    accumulator: A,
}

impl<P, S, A> Many<P, S, A> {
    pub fn new(parser: P, seed: S, accumulator: A) -> Self {
        Many {
            parser,
            seed,
            accumulator,
        }
    }
}

impl<P, S, A, O> Parser for Many<P, S, A>
where
    P: Parser,
    S: Fn(P::Output) -> O,
    A: Fn(O, P::Output) -> O,
{
    type Input = P::Input;
    type Output = O;
    type Error = P::Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        self.parser.parse(input).flat_map(|mut input, first_value| {
            let mut result = (self.seed)(first_value);
            loop {
                match self.parser.parse(input) {
                    ParseResult::Ok(i, value) => {
                        input = i;
                        result = (self.accumulator)(result, value);
                    }
                    ParseResult::None(i) => {
                        input = i;
                        break;
                    }
                    ParseResult::Err(i, err) => return ParseResult::Err(i, err),
                }
            }
            ParseResult::Ok(input, result)
        })
    }
}
