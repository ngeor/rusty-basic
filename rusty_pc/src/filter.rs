use crate::{ParseResult, Parser, SetContext};

pub trait Filter<I, C>: Parser<I, C>
where
    Self: Sized,
    Self::Error: Default,
    I: Clone,
{
    fn filter<F>(self, predicate: F) -> FilterParser<Self, F, Self::Error>
    where
        F: Fn(&Self::Output) -> bool,
    {
        FilterParser::new(self, predicate, None)
    }

    fn filter_or_err<F, E>(self, predicate: F, err: E) -> FilterParser<Self, F, E>
    where
        F: Fn(&Self::Output) -> bool,
        E: Clone,
        Self::Error: From<E>,
    {
        FilterParser::new(self, predicate, Some(err))
    }
}

impl<I, C, P> Filter<I, C> for P
where
    P: Parser<I, C>,
    P::Error: Default,
    I: Clone,
{
}

pub struct FilterParser<P, F, E> {
    parser: P,
    predicate: F,
    err_msg: Option<E>,
}

impl<P, F, E> FilterParser<P, F, E> {
    pub fn new(parser: P, predicate: F, err_msg: Option<E>) -> Self {
        Self {
            parser,
            predicate,
            err_msg,
        }
    }
}

impl<I, C, P, F, E> Parser<I, C> for FilterParser<P, F, E>
where
    I: Clone,
    P: Parser<I, C>,
    P::Error: Default + From<E>,
    F: Fn(&P::Output) -> bool,
    E: Clone,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        let original_input = input.clone();
        let (input, value) = self.parser.parse(input)?;
        if (self.predicate)(&value) {
            Ok((input, value))
        } else {
            let err = match &self.err_msg {
                Some(err) => P::Error::from(err.clone()),
                None => P::Error::default(),
            };

            Err((false, original_input, err))
        }
    }
}

impl<C, P, F, E> SetContext<C> for FilterParser<P, F, E>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
