use crate::{ParseResult, Parser, parser1_decl, parser1_impl};

parser1_decl!(
    trait ToOption {
        fn to_option();
    }
    struct ToOptionParser;
);

parser1_impl!(
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
