use crate::{
    pc::{ParseResult, ParseResultTrait, Parser},
    ParseError,
};

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

impl<I, P, S, A, O> Parser<I> for Many<P, S, A>
where
    P: Parser<I>,
    S: Fn(P::Output) -> O,
    A: Fn(O, P::Output) -> O,
{
    type Output = O;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, ParseError> {
        self.parser.parse(input).flat_map(|mut input, first_value| {
            let mut result = (self.seed)(first_value);
            loop {
                match self.parser.parse(input) {
                    Ok((i, value)) => {
                        input = i;
                        result = (self.accumulator)(result, value);
                    }
                    Err((false, i, _)) => {
                        input = i;
                        break;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Ok((input, result))
        })
    }
}
