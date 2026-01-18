use crate::{InputTrait, Parser, ParserErrorTrait, default_parse_error};

/// Creates a parser that can parse a list of element
/// separated by a delimiter.
pub trait DelimitedBy<I: InputTrait, C>: Parser<I, C> + Sized {
    /// Creates a new parser that uses this parser to parse elements
    /// that are separated by the given delimiter.
    ///
    /// Trailing delimiters are not allowed, in which case the given
    /// error will be returned (fatal error).
    ///
    /// # Arguments
    ///
    /// * self - The current parser, that is used to parse an element of the list
    /// * delimiter - THe parser that parses a delimiter (e.g. a comma, semicolon, etc.)
    /// * trailing_error - The error to return if a trailing delimiter is found
    fn delimited_by<D>(
        self,
        delimiter: D,
        trailing_error: Self::Error,
    ) -> DelimitedParser<Self, D, Self::Error, NormalElementCollector>
    where
        D: Parser<I, C, Error = Self::Error>,
    {
        debug_assert!(trailing_error.is_fatal());
        DelimitedParser {
            parser: self,
            delimiter,
            trailing_error,
            element_collector: NormalElementCollector,
        }
    }

    /// Creates a new parser that uses this parser to parse elements
    /// that are separated by the given delimiter.
    ///
    /// It is possible to have continuous delimiters with no element
    /// in between. That is why the output type is a Vec of Option elements.
    ///
    /// Trailing delimiters are not allowed, in which case the given
    /// error will be returned (fatal error).
    ///
    /// # Arguments
    ///
    /// * self - The current parser, that is used to parse an element of the list
    /// * delimiter - THe parser that parses a delimiter (e.g. a comma, semicolon, etc.)
    /// * trailing_error - The error to return if a trailing delimiter is found
    fn delimited_by_allow_missing<D>(
        self,
        delimiter: D,
        trailing_error: Self::Error,
    ) -> DelimitedParser<Self, D, Self::Error, OptionalElementCollector>
    where
        D: Parser<I, C, Error = Self::Error>,
    {
        debug_assert!(trailing_error.is_fatal());
        DelimitedParser {
            parser: self,
            delimiter,
            trailing_error,
            element_collector: OptionalElementCollector,
        }
    }
}

// blanket implementation of the DelimitedBy trait

impl<I, C, P> DelimitedBy<I, C> for P
where
    I: InputTrait,
    P: Parser<I, C>,
    P::Error: Clone + Default,
{
}

/// A parser that can parse a list of elements separated by a delimiter
pub struct DelimitedParser<P, D, E, A> {
    /// The parser that is used to parse the elements
    parser: P,
    /// The parser that is used to parse the delimiters
    delimiter: D,
    /// The error to return if a trailing delimiter is found
    trailing_error: E,
    /// Collects the elements into the final result
    element_collector: A,
}

/// Collects the elements into the final result
///
/// The type `E` is the type of the parsed elements.
pub trait ElementCollector<E> {
    /// The type of the element as it is collected.
    /// The normal case would be just the same as the `E` type,
    /// but if optional elements are allowed, it can be `Option<E>`.
    type CollectedElement;

    /// Maps the parsed element to the collected element.
    fn map(&self, element: E) -> Self::CollectedElement;

    /// Creates optionally a value for a missing element.
    /// This method should return `None` if missing elements
    /// are not supported.
    fn map_missing_element(&self) -> Option<Self::CollectedElement>;
}

/// Collects the elements into the final result,
/// without any transformation.
pub struct NormalElementCollector;

impl<E> ElementCollector<E> for NormalElementCollector {
    type CollectedElement = E;

    fn map(&self, element: E) -> E {
        element
    }

    fn map_missing_element(&self) -> Option<E> {
        // missing elements are not supported
        None
    }
}

/// Collects the elements into the final result,
/// but allows optional elements.
pub struct OptionalElementCollector;

impl<E> ElementCollector<E> for OptionalElementCollector {
    /// The collected element is an `Option<E>`
    /// in order to support optional elements.
    type CollectedElement = Option<E>;

    fn map(&self, element: E) -> Option<E> {
        // map the element to a Some
        Some(element)
    }

    fn map_missing_element(&self) -> Option<Option<E>> {
        // This can be a bit confusing,
        // as the signature is Option of Option of E.
        // Some indicates that we support missing elements,
        // and None is the missing element's value.
        Some(None)
    }
}

impl<I, C, P, D, E, A> Parser<I, C> for DelimitedParser<P, D, E, A>
where
    I: InputTrait,
    P: Parser<I, C, Error = E>,
    D: Parser<I, C, Error = E>,
    E: ParserErrorTrait,
    A: ElementCollector<P::Output>,
{
    // The output is determined by the element collector,
    // allowing some versatility of the output type,
    // without having to re-implement the parser,
    // or having to do a map on the output after parsing is done.
    type Output = Vec<A::CollectedElement>;
    type Error = E;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        let mut result: Self::Output = vec![];
        let mut state = State::Initial;
        let mut last_parsed = LastParsed::Nothing;
        loop {
            match self.parser.parse(input) {
                Ok(value) => {
                    // collect value
                    debug_assert!(state != State::AfterValue);
                    state = State::AfterValue;
                    result.push(self.element_collector.map(value));
                    last_parsed = LastParsed::Value;
                }
                Err(err) => {
                    if err.is_fatal() {
                        // always return on fatal errors
                        return Err(err);
                    }
                    // maybe get delimiter
                    state = State::AfterNoValue;
                }
            }

            debug_assert!(state == State::AfterValue || state == State::AfterNoValue);

            match self.delimiter.parse(input) {
                Ok(_) => {
                    if state == State::AfterNoValue {
                        match self.element_collector.map_missing_element() {
                            Some(value) => {
                                // push an optional missing element
                                result.push(value);
                            }
                            None => {
                                // missing elements are not allowed and we found a delimiter
                                return Err(self.trailing_error.clone());
                            }
                        }
                    }

                    state = State::AfterDelimiter;
                    last_parsed = LastParsed::Delimiter;
                }
                Err(err) => {
                    if err.is_fatal() {
                        // always return on fatal errors
                        return Err(err);
                    }
                    break;
                }
            }
        }

        match last_parsed {
            LastParsed::Nothing => default_parse_error(),
            LastParsed::Value => Ok(result),
            LastParsed::Delimiter => Err(self.trailing_error.clone()),
        }
    }
}

/// Keeps track of what was the most recently parsed item.
enum LastParsed {
    /// Nothing was parsed, this is the initial state.
    Nothing,

    /// The most recently parsed item was a value.
    Value,

    /// The most recently parsed item was a delimiter.
    Delimiter,
}

/// Keeps track of the state of the loop.
#[derive(PartialEq, Eq)]
enum State {
    /// Initial state.
    Initial,

    /// After having parsed a value.
    AfterValue,

    /// After having failed to parse a value.
    AfterNoValue,

    /// After having parsed a delimiter.
    AfterDelimiter,
}
