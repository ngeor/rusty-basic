use crate::{ParseResult, Parser, parser1_decl, parser1_impl};

parser1_decl!(
    trait OrDefault where Self::Output: Default;
    struct OrDefaultParser;
    fn or_default
);

parser1_impl!(
    impl Parser for OrDefaultParser where P::Output : Default {
        type Output = P::Output;

        Ok((input, value)) => Ok((input, value))
        Err((false, input, _)) => Ok((input, P::Output::default()))
    }
);
