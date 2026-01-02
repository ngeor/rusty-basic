use crate::{ParseResult, ParseResultTrait, Parser, parser_combinator};

parser_combinator!(
    trait FlatMap {
        fn flat_map<F, U>(mapper: F) -> U
        where
            F: Fn(I, Self::Output) -> ParseResult<I, U, Self::Error>
        ;
    }

    struct FlatMapParser<F>;

    fn parse<U>(&self, tokenizer) -> U
    where
        F: Fn(I, P::Output) -> ParseResult<I, U, P::Error>
    {
        self.parser.parse(tokenizer).flat_map(&self.mapper)
    }
);
