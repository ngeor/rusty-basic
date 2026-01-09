use crate::{OrDefault, ParseResult, ParseResultTrait, Parser};

pub trait Many<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn many<S, A, O>(
        self,
        seed: S,
        accumulator: A,
    ) -> impl Parser<I, C, Output = O, Error = Self::Error>
    where
        S: Fn(Self::Output) -> O,
        A: Fn(O, Self::Output) -> O,
        O: Default,
    {
        ManyParser::new(self, seed, accumulator)
    }
}

impl<I, C, P> Many<I, C> for P where P: Parser<I, C> {}

struct ManyParser<P, S, A> {
    parser: P,
    seed: S,
    accumulator: A,
}

impl<P, S, A> ManyParser<P, S, A> {
    pub fn new(parser: P, seed: S, accumulator: A) -> Self {
        Self {
            parser,
            seed,
            accumulator,
        }
    }
}

impl<I, C, P, S, A, O> Parser<I, C> for ManyParser<P, S, A>
where
    P: Parser<I, C>,
    S: Fn(P::Output) -> O,
    A: Fn(O, P::Output) -> O,
    O: Default,
{
    type Output = O;
    type Error = P::Error;
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
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

impl<C, P, S, A> crate::SetContext<C> for ManyParser<P, S, A>
where
    P: crate::SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}

pub trait ManyVec<I, C>: Many<I, C>
where
    Self: Sized,
{
    fn one_or_more(self) -> impl Parser<I, C, Output = Vec<Self::Output>, Error = Self::Error> {
        self.many(
            |e| vec![e],
            |mut v: Vec<Self::Output>, e| {
                v.push(e);
                v
            },
        )
    }

    fn zero_or_more(self) -> impl Parser<I, C, Output = Vec<Self::Output>, Error = Self::Error> {
        self.one_or_more().or_default()
    }
}

impl<I, C, P> ManyVec<I, C> for P where P: Many<I, C> {}
