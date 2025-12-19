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
    P::Input: Clone,
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
                let old_input = input.clone();
                match self.parser.parse(input) {
                    Ok((i, value)) => {
                        input = i;
                        result = (self.accumulator)(result, value);
                    }
                    Err((false, _)) => {
                        input = old_input;
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
