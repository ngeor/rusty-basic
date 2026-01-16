use crate::{ParseResult, Parser, SetContext};

pub trait Filter<I, C>: Parser<I, C>
where
    Self: Sized,
    Self::Error: Clone + Default,
    I: Clone,
{
    fn filter<F>(self, predicate: F) -> FilterParser<Self, F, Self::Error>
    where
        F: Fn(&Self::Output) -> bool,
    {
        FilterParser::new(self, predicate, None)
    }

    fn filter_or_err<F>(self, predicate: F, err: Self::Error) -> FilterParser<Self, F, Self::Error>
    where
        F: Fn(&Self::Output) -> bool,
    {
        FilterParser::new(self, predicate, Some(err))
    }
}

impl<I, C, P> Filter<I, C> for P
where
    P: Parser<I, C>,
    P::Error: Clone + Default,
    I: Clone,
{
}

pub struct FilterParser<P, F, E> {
    parser: P,
    predicate: F,
    err: Option<E>,
}

impl<P, F, E> FilterParser<P, F, E> {
    pub fn new(parser: P, predicate: F, err: Option<E>) -> Self {
        Self {
            parser,
            predicate,
            err,
        }
    }
}

impl<I, C, P, F, E> Parser<I, C> for FilterParser<P, F, E>
where
    P: Parser<I, C, Error = E>,
    E: Clone + Default,
    I: Clone,
    F: Fn(&P::Output) -> bool,
{
    type Output = P::Output;
    type Error = E;
    fn parse(&mut self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(tokenizer.clone()) {
            Ok((input, value)) => {
                if (self.predicate)(&value) {
                    Ok((input, value))
                } else {
                    let err = match &self.err {
                        Some(err) => err.clone(),
                        None => E::default(),
                    };

                    Err((false, tokenizer, err))
                }
            }
            Err(err) => Err(err),
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
