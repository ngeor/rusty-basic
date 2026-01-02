use crate::{ParseResult, Parser, default_parse_error, parser_combinator};

parser_combinator!(
    trait LoopWhile
    where Error: Default,
    {
        fn loop_while<F>(predicate: F) -> Vec<Self::Output>
        where
            F: Fn(&Self::Output) -> bool;
    }

    struct LoopWhileParser<F>;

    fn parse(&self, tokenizer) -> Vec<P::Output>
    where
        F: Fn(&P::Output) -> bool
    {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        let mut remaining = tokenizer;
        while keep_going {
            match self.parser.parse(remaining) {
                Ok((tokenizer, item)) => {
                    keep_going = (self.predicate)(&item);
                    // push to the list regardless
                    result.push(item);
                    remaining = tokenizer;
                }
                Err((false, i, _)) => {
                    remaining = i;
                    keep_going = false;
                }
                Err(err) => return Err(err),
            }
        }
        if result.is_empty() {
            default_parse_error(remaining)
        } else {
            Ok((remaining, result))
        }
    }
);
