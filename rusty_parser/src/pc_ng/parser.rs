use crate::pc_ng::and::And;
use crate::pc_ng::filter::Filter;
use crate::pc_ng::flat_map::FlatMap;
use crate::pc_ng::many::Many;
use crate::pc_ng::map::Map;
use crate::pc_ng::opt::Opt;
use crate::pc_ng::ParseResult;

pub trait Parser {
    type Input;
    type Output;
    type Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error>;

    fn and<R, F, O>(
        self,
        other: R,
        combiner: F,
    ) -> impl Parser<Input = Self::Input, Output = O, Error = Self::Error>
    where
        Self: Sized,
        R: Parser<Input = Self::Input, Error = Self::Error>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        And::new(self, other, combiner)
    }

    fn filter<F>(
        self,
        predicate: F,
    ) -> impl Parser<Input = Self::Input, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
        Self::Error: Default,
        F: Fn(&Self::Output) -> bool,
    {
        Filter::new(self, predicate)
    }

    fn flat_map<F, O>(
        self,
        mapper: F,
    ) -> impl Parser<Input = Self::Input, Output = O, Error = Self::Error>
    where
        Self: Sized,
        F: Fn(Self::Input, Self::Output) -> ParseResult<Self::Input, O, Self::Error>,
    {
        FlatMap::new(self, mapper)
    }

    fn many<S, A, O>(
        self,
        seed: S,
        accumulator: A,
    ) -> impl Parser<Input = Self::Input, Output = O, Error = Self::Error>
    where
        Self: Sized,
        Self::Input: Clone,
        S: Fn(Self::Output) -> O,
        A: Fn(O, Self::Output) -> O,
    {
        Many::new(self, seed, accumulator)
    }

    fn map<F, O>(
        self,
        mapper: F,
    ) -> impl Parser<Input = Self::Input, Output = O, Error = Self::Error>
    where
        Self: Sized,
        F: Fn(Self::Output) -> O,
    {
        Map::new(self, mapper)
    }

    fn opt(
        self,
    ) -> impl Parser<Input = Self::Input, Output = Option<Self::Output>, Error = Self::Error>
    where
        Self: Sized,
        Self::Input: Clone,
    {
        Opt::new(self)
    }
}
