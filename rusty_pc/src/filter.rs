use crate::{InputTrait, Parser, ParserErrorTrait, SetContext};

pub struct FilterParser<P, F> {
    parser: P,
    predicate: F,
}

impl<P, F> FilterParser<P, F> {
    pub(crate) fn new(parser: P, predicate: F) -> Self {
        Self { parser, predicate }
    }
}

impl<I, C, P, F> Parser<I, C> for FilterParser<P, F>
where
    I: InputTrait,
    I: InputTrait,
    P: Parser<I, C>,
    F: FilterPredicate<P::Output, P::Error>,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        let original_input = input.get_position();
        let value = self.parser.parse(input)?;
        match self.predicate.filter(value) {
            Ok(value) => Ok(value),
            Err(err) => {
                if err.is_soft() {
                    input.set_position(original_input);
                }
                Err(err)
            }
        }
    }
}

impl<C, P, F> SetContext<C> for FilterParser<P, F>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}

pub trait FilterPredicate<T, E> {
    fn filter(&self, value: T) -> Result<T, E>;
}

impl<F, T, E> FilterPredicate<T, E> for F
where
    F: Fn(&T) -> bool,
    E: ParserErrorTrait,
{
    fn filter(&self, value: T) -> Result<T, E> {
        if (self)(&value) {
            Ok(value)
        } else {
            Err(E::default())
        }
    }
}
