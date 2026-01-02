use crate::{ParseResult, ParseResultTrait, Parser, parser_combinator};

parser_combinator!(
    trait Map {
        fn map<F, U>(mapper: F) -> U
        where
            F: Fn(Self::Output) -> U;
    }

    struct MapParser<F>;

    fn parse<U>(&self, tokenizer) -> U
    where
        F: Fn(P::Output) -> U
    {
        self.parser.parse(tokenizer).map_ok(&self.mapper)
    }
);
