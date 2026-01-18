use std::marker::PhantomData;

use crate::{InputTrait, Parser, ParserErrorTrait, SetContext, Token};

/// Collects multiple values from the underlying parser as long as parsing succeeds.
pub trait Many<I: InputTrait, C>: Parser<I, C>
where
    Self: Sized,
{
    /// Collects multiple values from the underlying parser as long as parsing succeeds.
    /// The combiner trait combines the multiple values into the final result.
    fn many<F, O>(self, combiner: F) -> ManyParser<Self, F, O>
    where
        F: ManyCombiner<Self::Output, O>,
        O: Default,
    {
        ManyParser::new(self, combiner, false)
    }

    fn many_allow_none<F, O>(self, combiner: F) -> ManyParser<Self, F, O>
    where
        F: ManyCombiner<Self::Output, O>,
        O: Default,
    {
        ManyParser::new(self, combiner, true)
    }
}

impl<I, C, P> Many<I, C> for P
where
    I: InputTrait,
    P: Parser<I, C>,
{
}

pub struct ManyParser<P, F, O> {
    parser: P,
    combiner: F,
    allow_none: bool,
    _marker: PhantomData<O>,
}

impl<P, F, O> ManyParser<P, F, O> {
    pub fn new(parser: P, combiner: F, allow_none: bool) -> Self {
        Self {
            parser,
            combiner,
            allow_none,
            _marker: PhantomData,
        }
    }
}

impl<I, C, P, F, O> Parser<I, C> for ManyParser<P, F, O>
where
    I: InputTrait,
    P: Parser<I, C>,
    F: ManyCombiner<P::Output, O>,
    O: Default,
{
    type Output = O;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(first_value) => {
                let mut result = self.combiner.seed(first_value);
                loop {
                    match self.parser.parse(input) {
                        Ok(value) => {
                            result = self.combiner.accumulate(result, value);
                        }
                        Err(err) if !err.is_fatal() => {
                            break;
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
                Ok(result)
            }
            Err(err) if !err.is_fatal() => {
                if self.allow_none {
                    Ok(O::default())
                } else {
                    Err(err)
                }
            }
            Err(err) => Err(err),
        }
    }
}

impl<C, P, F, O> SetContext<C> for ManyParser<P, F, O>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}

pub trait ManyVec<I, C>: Many<I, C>
where
    Self: Sized,
    I: InputTrait,
{
    fn one_or_more(self) -> ManyParser<Self, VecManyCombiner, Vec<Self::Output>> {
        self.many(VecManyCombiner)
    }

    fn zero_or_more(self) -> ManyParser<Self, VecManyCombiner, Vec<Self::Output>> {
        self.many_allow_none(VecManyCombiner)
    }
}

impl<I, C, P> ManyVec<I, C> for P
where
    P: Many<I, C>,
    I: InputTrait,
{
}

/// Combines multiple values into a single result.
///
/// * `E` is the type of the parsed elements
/// * `O` is the type of the final result
pub trait ManyCombiner<E, O> {
    /// Creates the initial result value out of the first successfully parsed element.
    fn seed(&self, element: E) -> O;

    /// Accumulates subsequent parsed values into the result that has been parsed so far.
    fn accumulate(&self, result: O, element: E) -> O;
}

/// Combines multiple values into a String.
pub struct StringManyCombiner;

impl ManyCombiner<char, String> for StringManyCombiner {
    fn seed(&self, element: char) -> String {
        String::from(element)
    }

    fn accumulate(&self, mut result: String, element: char) -> String {
        result.push(element);
        result
    }
}

impl ManyCombiner<Token, String> for StringManyCombiner {
    fn seed(&self, element: Token) -> String {
        element.text()
    }

    fn accumulate(&self, mut result: String, element: Token) -> String {
        result.push_str(element.as_str());
        result
    }
}

/// Combines multiple values into a Vec.
pub struct VecManyCombiner;

impl<E> ManyCombiner<E, Vec<E>> for VecManyCombiner {
    fn seed(&self, element: E) -> Vec<E> {
        vec![element]
    }

    fn accumulate(&self, mut result: Vec<E>, element: E) -> Vec<E> {
        result.push(element);
        result
    }
}

/// Combines multiple values into an empty result.
/// Useful when the collected values will be ignored.
pub struct IgnoringManyCombiner;

impl<E> ManyCombiner<E, ()> for IgnoringManyCombiner {
    fn seed(&self, _element: E) -> () {
        ()
    }

    fn accumulate(&self, _result: (), _element: E) -> () {
        ()
    }
}
