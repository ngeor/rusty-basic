use crate::{ParseResult, Parser, parser1};

parser1!(
    trait ToOption {
        fn to_option();
    }

    impl Parser for ToOptionParser {
        type Output = Option<P::Output>;

        fn parse(&self, input) {
            match self.parser.parse(input) {
                Ok((input, value)) => Ok((input, Some(value))),
                Err((false, input, _)) => Ok((input, None)),
                Err(err) => Err(err)
            }
        }
    }
);
