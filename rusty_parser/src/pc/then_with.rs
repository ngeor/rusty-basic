use crate::pc::{ParseResult, ParseResultTrait, Parser};

/// Similar to `and_without_undo`, but the right parser is created dynamically
/// based on the result of the first parser.
pub trait ThenWith<I>: Parser<I>
where
    Self: Sized,
{
    /// Similar to `and_without_undo`, but the right parser is created dynamically
    /// based on the result of the first parser.
    fn then_with<RF, R, F, O>(
        self,
        right_factory: RF,
        combiner: F,
    ) -> impl Parser<I, Output = O, Error = Self::Error>
    where
        RF: Fn(&Self::Output) -> R,
        R: Parser<I, Error = Self::Error>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        ThenWithParser::new(self, right_factory, combiner)
    }
}

impl<I, P> ThenWith<I> for P where P: Parser<I> {}

struct ThenWithParser<L, R, F> {
    left: L,
    right: R,
    combiner: F,
}
impl<L, R, F> ThenWithParser<L, R, F> {
    pub fn new(left: L, right: R, combiner: F) -> Self {
        Self {
            left,
            right,
            combiner,
        }
    }
}

impl<I, L, RF, R, F, O> Parser<I> for ThenWithParser<L, RF, F>
where
    L: Parser<I>,
    RF: Fn(&L::Output) -> R,
    R: Parser<I, Error = L::Error>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Output = O;
    type Error = L::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.left.parse(tokenizer).flat_map(|tokenizer: I, first| {
            let right_parser = (self.right)(&first);
            right_parser
                .parse(tokenizer)
                .map_ok(|r| (self.combiner)(first, r))
        })
    }
}
