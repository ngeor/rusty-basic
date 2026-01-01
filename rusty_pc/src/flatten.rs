use crate::{ParseResult, Parser, parser_combinator};

parser_combinator!(
    trait Flatten {
        fn flatten() -> <Self::Output as Parser<I, C>>::Output
        where Self::Output : Parser<I, C, Error = Self::Error>
        ;
    }

    struct FlattenParser;

    fn parse(&self, input) -> <P::Output as Parser<I, C>>::Output
    where P::Output : Parser<I, C, Error = P::Error>
    {
        match self.parser.parse(input) {
            Ok((i, new_parser)) => new_parser.parse(i),
            Err(err) => Err(err),
        }
    }
);
