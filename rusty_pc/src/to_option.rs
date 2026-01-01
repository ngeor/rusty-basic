use crate::{ParseResult, Parser, parser1_decl, parser1_impl};

parser1_decl!(
    trait ToOption;
    struct ToOptionParser;
    fn to_option
);

parser1_impl!(
    impl Parser for ToOptionParser {
        type Output = Option<P::Output>;

        Ok((input, value)) => Ok((input, Some(value)))
        Err((false, input, _)) => Ok((input, None))
    }
);
