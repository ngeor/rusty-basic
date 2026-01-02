use crate::{OrDefault, ParseResult, ParseResultTrait, Parser, parser_combinator};

parser_combinator!(
    trait Many {
        fn many<S, A, O>(seed: S, accumulator: A) -> O
        where S : Fn(Self::Output) -> O,
              A : Fn(O, Self::Output) -> O,
              O : Default;
    }

    struct ManyParser<S, A>;

    fn parse<O>(&self, input) -> O
    where S : Fn(P::Output) -> O,
          A : Fn(O, P::Output) -> O,
          O : Default
    {
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
);

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
