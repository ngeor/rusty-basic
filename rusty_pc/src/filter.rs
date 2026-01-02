use crate::{ParseResult, Parser, default_parse_error, parser_combinator};

parser_combinator!(
    trait Filter
    where
        I: Clone,
        Error: Default,
    {
        fn filter<F>(predicate: F)
        where
            F: Fn(&Self::Output) -> bool;
    }

    struct FilterParser<F>;

    fn parse(&self, tokenizer)
    where
        F: Fn(&P::Output) -> bool {
        match self.parser.parse(tokenizer.clone()) {
            Ok((input, value)) => {
                if (self.predicate)(&value) {
                    Ok((input, value))
                } else {
                    // return original input here
                    default_parse_error(tokenizer)
                }
            }
            Err(err) => Err(err),
        }
    }
);
