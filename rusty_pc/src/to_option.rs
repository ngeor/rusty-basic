use crate::{ParseResult, Parser, parser_combinator};

parser_combinator!(
    trait ToOption {
        fn to_option() -> Option<Self::Output>;
    }

    struct ToOptionParser;

    fn parse(&self, input) -> Option<P::Output> {
        match self.parser.parse(input) {
            Ok((input,value)) => Ok((input,Some(value))),
            Err((false,input,_)) => Ok((input,None)),
            Err(err) => Err(err)
        }
    }
);
