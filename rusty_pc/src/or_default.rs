use crate::{ParseResult, Parser, parser1};

parser1!(
    trait OrDefault
    where
        Self::Output: Default,
    {
        fn or_default();
    }

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
