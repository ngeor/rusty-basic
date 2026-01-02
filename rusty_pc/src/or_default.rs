use crate::{ParseResult, Parser, parser_combinator};

parser_combinator!(
    trait OrDefault where Output: Default, {
        fn or_default();
    }

    struct OrDefaultParser;

    fn parse(&self, input) {
        match self.parser.parse(input) {
            Ok((input,value)) => Ok((input,value)),
            Err((false,input,_)) => Ok((input,P::Output::default())),
            Err(err) => Err(err)
        }
    }
);
