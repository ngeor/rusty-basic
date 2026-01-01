use crate::{
    ParseResult, ParseResultTrait, Parser, default_parse_error, parser1_decl, parser1_impl
};

parser1_decl!(
    trait FilterMap
    where
        Self::Error: Default,
        I: Clone,
    {
        fn filter_map<F, U>(predicate: F)
        where
            F: Fn(&Self::Output) -> Option<U>;
    }

    struct FilterMapParser<F>;
);

parser1_impl!(
    impl<U> Parser for FilterMapParser<F>
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
