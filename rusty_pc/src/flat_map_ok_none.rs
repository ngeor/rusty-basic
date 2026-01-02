use crate::{ParseResult, Parser, parser_combinator};

parser_combinator!(
    trait FlatMapOkNone {
        /// Flat map the result of this parser for successful and incomplete results.
        /// Mapping is done by the given closures.
        /// Other errors are never allowed to be re-mapped.
        fn flat_map_ok_none<F, G, U>(ok_mapper: F, incomplete_mapper: G) -> U
        where F : Fn(I, Self::Output) -> ParseResult<I, U, Self::Error>,
              G : Fn(I) -> ParseResult<I, U, Self::Error>;
    }

    struct FlatMapOkNoneParser<F, G>;

    fn parse<U>(&self, input) -> U
    where F : Fn(I, P::Output) -> ParseResult<I, U, P::Error>,
              G : Fn(I) -> ParseResult<I, U, P::Error>
    {
        match self.parser.parse(input) {
            Ok((input, value)) => (self.ok_mapper)(input, value),
            Err((false, i, _)) => (self.incomplete_mapper)(i),
            Err(err) => Err(err),
        }
    }
);

pub trait FlatMapNegateNone<I, C>: FlatMapOkNone<I, C>
where
    Self: Sized,
{
    /// Flat map the successful of this parser into an empty result.
    /// The Failed result is negated and mapped into an empty successful result (i.e. `None` becomes `Ok(())`).
    fn flat_map_negate_none<F>(
        self,
        ok_mapper: F,
    ) -> impl Parser<I, C, Output = (), Error = Self::Error>
    where
        F: Fn(I, Self::Output) -> ParseResult<I, (), Self::Error>,
    {
        self.flat_map_ok_none(ok_mapper, |input| Ok((input, ())))
    }
}

impl<I, C, P> FlatMapNegateNone<I, C> for P where P: FlatMapOkNone<I, C> {}
