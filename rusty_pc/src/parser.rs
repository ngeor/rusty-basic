use crate::and::{AndParser, Combiner, KeepLeftCombiner, KeepRightCombiner, TupleCombiner};
use crate::and_then::AndThenParser;
use crate::and_then_err::AndThenErrParser;
use crate::boxed::BoxedParser;
use crate::delimited::{DelimitedParser, NormalElementCollector, OptionalElementCollector};
use crate::filter::{FilterParser, FilterPredicate};
use crate::filter_map::FilterMapParser;
use crate::flatten::FlattenParser;
use crate::many::{ManyCombiner, ManyParser, VecManyCombiner};
use crate::map::MapParser;
use crate::map_err::{
    ErrorMapper, FatalErrorOverrider, MapErrParser, SoftErrorOverrider, ToFatalErrorMapper
};
use crate::or_default::OrDefaultParser;
use crate::peek::PeekParser;
use crate::to_option::ToOptionParser;
use crate::{InitContextParser, NoContextParser, ThenWithContextParser};

/// A parser uses the given input in order to produce a result.
pub trait Parser<I, C = ()>
where
    I: InputTrait,
{
    type Output;
    type Error: ParserErrorTrait;

    /// Parses the given input and returns a result.
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error>;

    // =======================================================================
    // And
    // =======================================================================

    /// Parses both the left and the right side.
    /// If the right side fails with a soft error, parsing of the left side is undone.
    fn and<R, F, O>(self, right: R, combiner: F) -> AndParser<Self, R, F, O>
    where
        Self: Sized,
        R: Parser<I, C, Error = Self::Error>,
        F: Combiner<Self::Output, R::Output, O>,
    {
        AndParser::new(self, right, combiner)
    }

    fn and_tuple<R>(self, right: R) -> AndParser<Self, R, TupleCombiner, (Self::Output, R::Output)>
    where
        Self: Sized,
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and(right, TupleCombiner)
    }

    fn and_keep_left<R>(self, right: R) -> AndParser<Self, R, KeepLeftCombiner, Self::Output>
    where
        Self: Sized,
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and(right, KeepLeftCombiner)
    }

    fn and_keep_right<R>(self, right: R) -> AndParser<Self, R, KeepRightCombiner, R::Output>
    where
        Self: Sized,
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and(right, KeepRightCombiner)
    }

    /// Parses the left side and optionally the right side.
    /// The combiner function maps the left and (optional) right output to the final result.
    fn and_opt<R, F, O>(self, right: R, combiner: F) -> AndParser<Self, ToOptionParser<R>, F, O>
    where
        Self: Sized,
        R: Parser<I, C, Error = Self::Error>,
        F: Combiner<Self::Output, Option<R::Output>, O>,
    {
        self.and(right.to_option(), combiner)
    }

    /// Parses the left side and optionally the right side.
    /// The result is a tuple of both sides.
    fn and_opt_tuple<R>(
        self,
        right: R,
    ) -> AndParser<Self, ToOptionParser<R>, TupleCombiner, (Self::Output, Option<R::Output>)>
    where
        Self: Sized,
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and_opt(right, TupleCombiner)
    }

    /// Parses the left side and optionally the right side.
    /// The result is only the left side's output.
    fn and_opt_keep_left<R>(
        self,
        right: R,
    ) -> AndParser<Self, ToOptionParser<R>, KeepLeftCombiner, Self::Output>
    where
        Self: Sized,
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and_opt(right, KeepLeftCombiner)
    }

    /// Parses the left side and optionally the right side.
    /// The result is only the right side's output.
    fn and_opt_keep_right<R>(
        self,
        right: R,
    ) -> AndParser<Self, ToOptionParser<R>, KeepRightCombiner, Option<R::Output>>
    where
        Self: Sized,
        R: Parser<I, C, Error = Self::Error>,
    {
        self.and_opt(right, KeepRightCombiner)
    }

    // =======================================================================
    // AndThen
    // =======================================================================

    /// Creates a new parser that maps the successful result of this parser
    /// with the given function that returns a new result.
    ///
    /// Note that even if the mapper function returns a soft error,
    /// the input is not backtracked to the original position.
    fn and_then<F, U>(self, mapper: F) -> AndThenParser<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> Result<U, Self::Error>,
    {
        AndThenParser::new(self, mapper)
    }

    // =======================================================================
    // AndThenErr
    // =======================================================================

    /// Creates a new parser that maps the sort error result of this parser
    /// with the given function that returns a new result.
    ///
    /// Note that even if the mapper function returns a soft error,
    /// the input is not backtracked to the original position.
    fn and_then_err<F>(self, mapper: F) -> AndThenErrParser<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> Result<Self::Output, Self::Error>,
    {
        AndThenErrParser::new(self, mapper)
    }

    // =======================================================================
    // Boxed
    // =======================================================================

    fn boxed(self) -> BoxedParser<I, C, Self::Output, Self::Error>
    where
        Self: Sized + 'static,
    {
        BoxedParser::new(self)
    }

    // =======================================================================
    // Delimited
    // =======================================================================

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
        Self: Sized,
        D: Parser<I, C, Error = Self::Error>,
    {
        DelimitedParser::new(self, delimiter, trailing_error, NormalElementCollector)
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
        Self: Sized,
        D: Parser<I, C, Error = Self::Error>,
    {
        DelimitedParser::new(self, delimiter, trailing_error, OptionalElementCollector)
    }

    // =======================================================================
    // Filter
    // =======================================================================

    fn filter<F>(self, predicate: F) -> FilterParser<Self, F>
    where
        Self: Sized,
        F: FilterPredicate<Self::Output, Self::Error>,
    {
        FilterParser::new(self, predicate)
    }

    // =======================================================================
    // FilterMap
    // =======================================================================

    fn filter_map<F, U>(self, predicate: F) -> FilterMapParser<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> Option<U>,
    {
        FilterMapParser::new(self, predicate)
    }

    // =======================================================================
    // Flatten
    // =======================================================================

    fn flatten(self) -> FlattenParser<Self>
    where
        Self: Sized,
        Self::Output: Parser<I, C, Error = Self::Error>,
    {
        FlattenParser::new(self)
    }

    // =======================================================================
    // InitContext
    // =======================================================================

    /// Creates a parser that will initialize the context of the underlying parser
    /// to the given value before parsing starts.
    fn init_context(self, value: C) -> InitContextParser<Self, C>
    where
        Self: Sized,
        C: Clone,
    {
        InitContextParser::new(self, value)
    }

    // =======================================================================
    // Many
    // =======================================================================

    /// Collects multiple values from the underlying parser as long as parsing succeeds.
    /// The combiner trait combines the multiple values into the final result.
    fn many<F, O>(self, combiner: F) -> ManyParser<Self, F, O>
    where
        Self: Sized,
        F: ManyCombiner<Self::Output, O>,
        O: Default,
    {
        ManyParser::new(self, combiner, false)
    }

    fn many_allow_none<F, O>(self, combiner: F) -> ManyParser<Self, F, O>
    where
        Self: Sized,
        F: ManyCombiner<Self::Output, O>,
        O: Default,
    {
        ManyParser::new(self, combiner, true)
    }

    fn one_or_more(self) -> ManyParser<Self, VecManyCombiner, Vec<Self::Output>>
    where
        Self: Sized,
    {
        self.many(VecManyCombiner)
    }

    fn zero_or_more(self) -> ManyParser<Self, VecManyCombiner, Vec<Self::Output>>
    where
        Self: Sized,
    {
        self.many_allow_none(VecManyCombiner)
    }

    // =======================================================================
    // Map
    // =======================================================================

    fn map<F, U>(self, mapper: F) -> MapParser<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U,
    {
        MapParser::new(self, mapper)
    }

    // =======================================================================
    // MapErr
    // =======================================================================

    /// Maps the error of this parser.
    /// If the parser is successful, the value is returned as-is.
    /// If the parser returns an error, the given mapper is used to map the error
    /// to a new error.
    fn map_err<F>(self, mapper: F) -> MapErrParser<Self, F>
    where
        Self: Sized,
        F: ErrorMapper<Self::Error>,
    {
        MapErrParser::new(self, mapper)
    }

    /// If this parser returns a soft error, the soft error will be replaced by
    /// the given error (which might be soft or fatal).
    fn with_soft_err(self, err: Self::Error) -> MapErrParser<Self, SoftErrorOverrider<Self::Error>>
    where
        Self: Sized,
    {
        self.map_err(SoftErrorOverrider::new(err))
    }

    /// If this parser returns a soft error, the soft error will be replaced by
    /// the given error, which must be fatal.
    fn or_fail(self, err: Self::Error) -> MapErrParser<Self, SoftErrorOverrider<Self::Error>>
    where
        Self: Sized,
    {
        assert!(err.is_fatal());
        self.with_soft_err(err)
    }

    /// If this parser returns a fatal error, the fatal error will be replaced by the given error
    /// (which must be fatal too).
    fn with_fatal_err(
        self,
        err: Self::Error,
    ) -> MapErrParser<Self, FatalErrorOverrider<Self::Error>>
    where
        Self: Sized,
    {
        assert!(err.is_fatal());
        self.map_err(FatalErrorOverrider::new(err))
    }

    /// If this parser returns a soft error, it will be converted to a fatal error.
    fn to_fatal(self) -> MapErrParser<Self, ToFatalErrorMapper>
    where
        Self: Sized,
    {
        self.map_err(ToFatalErrorMapper)
    }

    // =======================================================================
    // NoContext
    // =======================================================================

    fn no_context<C2>(self) -> NoContextParser<Self, C, C2>
    where
        Self: Sized,
    {
        NoContextParser::new(self)
    }

    // =======================================================================
    // OrDefault
    // =======================================================================

    fn or_default(self) -> OrDefaultParser<Self>
    where
        Self: Sized,
    {
        OrDefaultParser::new(self)
    }

    // =======================================================================
    // Peek
    // =======================================================================

    fn peek(self) -> PeekParser<Self>
    where
        Self: Sized,
    {
        PeekParser::new(self)
    }

    // =======================================================================
    // ThenWithInContext
    // =======================================================================

    /// Combines this parser with another parser,
    /// setting the other parser's context before parsing
    /// based on the result of this parser.
    ///
    /// The right-side parser is treated as a 'complete' parser,
    /// i.e. soft errors will be converted to fatal.
    ///
    /// # Arguments
    ///
    /// * self: this parser (the left-side parser)
    /// * other: the right-side parser
    /// * ctx_projection: a function that maps the left-side result into the right-side context
    fn then_with_in_context<R, F, A, O, CR>(
        self,
        other: R,
        ctx_projection: F,
        combiner: A,
    ) -> ThenWithContextParser<Self, R, F, A, O>
    where
        Self: Sized,
        R: Parser<I, CR, Error = Self::Error>,
        F: Fn(&Self::Output) -> CR,
        A: Combiner<Self::Output, R::Output, O>,
    {
        ThenWithContextParser::new(self, other, ctx_projection, combiner)
    }

    // =======================================================================
    // ToOption
    // =======================================================================

    fn to_option(self) -> ToOptionParser<Self>
    where
        Self: Sized,
    {
        ToOptionParser::new(self)
    }
}

pub trait InputTrait {
    type Output;

    /// Returns the next element.
    /// Does not advance the position.
    fn peek(&self) -> Self::Output;

    /// Returns the next element.
    /// Advances the position.
    fn read(&mut self) -> Self::Output;

    /// Gets the current position within the source.
    fn get_position(&self) -> usize;

    /// Is the input at the end of file.
    fn is_eof(&self) -> bool;

    /// Sets the current position within the source.
    fn set_position(&mut self, position: usize);
}

pub trait ParserErrorTrait: Clone + Default {
    /// Gets a value indicating whether this is a fatal error or not.
    /// Returns true if the error is fatal, false is the error is soft.
    fn is_fatal(&self) -> bool;

    /// Gets a value indicating whether this is a soft error or not.
    /// Returns true if the error is soft, false is the error is fatal.
    fn is_soft(&self) -> bool {
        !self.is_fatal()
    }

    /// Converts this error into a fatal.
    fn to_fatal(self) -> Self;
}

/// Creates a failed result containing the default parse error (soft).
pub fn default_parse_error<O, E>() -> Result<O, E>
where
    E: ParserErrorTrait,
{
    Err(E::default())
}
