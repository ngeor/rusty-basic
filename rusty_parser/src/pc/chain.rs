use crate::error::ParseError;
use crate::pc::{ParseResult, ParseResultTrait, Parser};

pub trait Chain<I>: Parser<I>
where
    Self: Sized,
{
    fn chain<RF, R, F, O>(self, right_factory: RF, combiner: F) -> impl Parser<I, Output = O>
    where
        RF: Fn(&Self::Output) -> R,
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        ChainParser::new(self, right_factory, combiner)
    }
}

impl<I, P> Chain<I> for P where P: Parser<I> {}

struct ChainParser<L, R, F> {
    left: L,
    right: R,
    combiner: F,
}
impl<L, R, F> ChainParser<L, R, F> {
    pub fn new(left: L, right: R, combiner: F) -> Self {
        Self {
            left,
            right,
            combiner,
        }
    }
}

impl<I, L, RF, R, F, O> Parser<I> for ChainParser<L, RF, F>
where
    L: Parser<I>,
    RF: Fn(&L::Output) -> R,
    R: Parser<I>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Output = O;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        self.left.parse(tokenizer).flat_map(|tokenizer: I, first| {
            let right_parser = (self.right)(&first);
            right_parser
                .parse(tokenizer)
                .map_ok(|r| (self.combiner)(first, r))
        })
    }
}
