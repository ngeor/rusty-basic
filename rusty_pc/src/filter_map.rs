use crate::{ParseResult, ParseResultTrait, Parser, default_parse_error, parser1};

parser1!(
    trait FilterMap
    where
        Self::Error: Default,
        I: Clone,
    {
        fn filter_map<F, U>(predicate: F)
        where
            F: Fn(&Self::Output) -> Option<U>;
    }

    impl Parser for FilterMapParser<F>
    where
        P::Error: Default,
        I: Clone,
        F: Fn(&P::Output) -> Option<U>
    {
        type Output = U;

        fn parse(&self, tokenizer) {
            self.parser
            .parse(tokenizer.clone())
            .flat_map(|input, result| match (self.predicate)(&result) {
                Some(value) => Ok((input, value)),
                None => default_parse_error(tokenizer),
            })
        }
    }
);
