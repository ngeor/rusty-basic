use crate::{ParseResult, ParseResultTrait, Parser, default_parse_error, parser_combinator};

parser_combinator!(
    trait FilterMap
    where
        I: Clone,
        Error: Default,
    {
        fn filter_map<F, U>(predicate: F) -> U
        where
            F: Fn(&Self::Output) -> Option<U>;
    }

    struct FilterMapParser<F>;

    fn parse<U>(&self, tokenizer) -> U
    where
        F: Fn(&P::Output) -> Option<U>
    {
        self.parser
        .parse(tokenizer.clone())
        .flat_map(|input, result| match (self.predicate)(&result) {
            Some(value) => Ok((input, value)),
            None => default_parse_error(tokenizer),
        })
    }
);
