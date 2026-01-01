use crate::{ParseResult, Parser, parser1_decl, parser1_impl};

parser1_decl!(
    trait OrDefault
    where
        Self::Output: Default,
    {
        fn or_default();
    }
    struct OrDefaultParser;
);

parser1_impl!(
    impl Parser for OrDefaultParser where P::Output : Default {
        type Output = P::Output;

        fn parse(&self, input) {
            match self.parser.parse(input) {
                Ok((input, value)) => Ok((input, value)),
                Err((false, input, _)) => Ok((input, P::Output::default())),
                Err(err) => Err(err)
            }
        }
    }
);
