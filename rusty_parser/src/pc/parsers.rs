use crate::pc::and::*;
use crate::pc::flat_map::*;
use crate::pc::many::*;
use crate::pc::map::*;
use crate::pc::{
    AllowNoneIfParser, ChainParser, FilterMapParser, FilterParser, LoopWhile, MessageProvider,
    NoIncompleteParser, OrFailParser, OrParser, ParseResult, SurroundParser, Tokenizer, Undo,
    WithExpectedMessage,
};
use crate::ParseError;

// TODO make QError generic param too

/// A parser uses a [Tokenizer] in order to produce a result.
///
/// There are two different types of failures:
/// - incomplete: another parser might be able to succeed
/// - fatal: all parsing should stop
pub trait Parser<I: Tokenizer + 'static> {
    type Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError>;

    /**
     * And
     */

    /// Parses both the left and the right side.
    /// If the right side fails, parsing of the left side is undone.
    fn and<R>(self, right: R) -> impl Parser<I, Output = (Self::Output, R::Output)>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
    {
        AndPC::new(self, right)
    }

    /**
     * And Without Undo
     */

    /// Parses both the left and the right side.
    /// Be careful: if the right side fails, parsing of the left side
    /// is not undone. This should not be used unless it's certain
    /// that the right side can't fail.
    fn and_without_undo<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        Self: Sized,
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        AndWithoutUndoPC::new(self, right, combiner)
    }

    /// Parses the left side and optionally the right side.
    fn and_opt<R>(self, right: R) -> impl Parser<I, Output = (Self::Output, Option<R::Output>)>
    where
        Self: Sized,
        R: Parser<I>,
    {
        self.and_without_undo(right.to_option(), |left, right| (left, right))
    }

    /// Parses the left side and returns the right side.
    /// If the left does not succeed, the right is not parsed.
    /// Be careful: If the right does not succeed, the left is not undone.
    /// This should not be used unless it's certain that the right can't fail.
    /// TODO use a NonOptParser here for the right side.
    fn then_demand<R>(self, right: R) -> impl Parser<I, Output = R::Output>
    where
        Self: Sized,
        R: Parser<I>,
    {
        self.and_without_undo(right, |_, right| right)
    }

    /**
     * Map
     */

    fn map<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U,
    {
        MapPC::new(self, mapper)
    }

    fn keep_left<L, R>(self) -> impl Parser<I, Output = L>
    where
        Self: Sized + Parser<I, Output = (L, R)>,
    {
        self.map(|(l, _)| l)
    }

    fn keep_right<L, R>(self) -> impl Parser<I, Output = R>
    where
        Self: Sized + Parser<I, Output = (L, R)>,
    {
        self.map(|(_, r)| r)
    }

    /// Map the result of this parser for successful and incomplete results.
    /// The given mapper implements [MapOk] which takes care of the mapping.
    fn map_ok_trait<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: MapOk<Self::Output, U>,
    {
        MapOkNoneTraitPC::new(self, mapper)
    }

    fn to_option(self) -> impl Parser<I, Output = Option<Self::Output>>
    where
        Self: Sized,
    {
        self.map_ok_trait(MapToOption)
    }

    fn or_default(self) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
        Self::Output: Default,
    {
        self.map_ok_trait(MapToDefault)
    }

    /**
     * Flat Map
     */

    /// Flat map the result of this parser for successful results.
    fn flat_map<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> ParseResult<U, ParseError>,
    {
        FlatMapPC::new(self, mapper)
    }

    /// Flat map the result of this parser for successful and incomplete results.
    /// Mapping is done by the given closures.
    /// Other errors are never allowed to be re-mapped.
    /// TODO: add some common helpers for incomplete_mapper == Ok(()) and incomplete_mapper == None
    fn flat_map_ok_none_closures<F, G, U>(
        self,
        ok_mapper: F,
        incomplete_mapper: G,
    ) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> ParseResult<U, ParseError>,
        G: Fn() -> ParseResult<U, ParseError>,
    {
        FlatMapOkNoneClosuresPC::new(self, ok_mapper, incomplete_mapper)
    }

    /**
     * Not reviewed yet
     */

    fn filter<F>(self, predicate: F) -> FilterParser<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> bool,
        Self::Output: Undo,
    {
        FilterParser::new(self, predicate)
    }

    fn filter_map<F, U>(self, mapper: F) -> FilterMapParser<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> Option<U>,
        Self::Output: Undo,
    {
        FilterMapParser::new(self, mapper)
    }

    fn loop_while<F>(self, predicate: F) -> LoopWhile<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> bool,
    {
        LoopWhile::new(self, predicate)
    }

    fn with_expected_message<F>(self, f: F) -> WithExpectedMessage<Self, F>
    where
        Self: Sized,
        F: MessageProvider,
    {
        WithExpectedMessage::new(self, f)
    }

    #[deprecated]
    fn or_fail(self, err: ParseError) -> OrFailParser<Self>
    where
        Self: Sized,
    {
        OrFailParser::new(self, err)
    }

    #[deprecated]
    fn no_incomplete(self) -> NoIncompleteParser<Self>
    where
        Self: Sized,
    {
        NoIncompleteParser::new(self)
    }

    fn or<O, R>(self, right: R) -> OrParser<I, O>
    where
        Self: Sized + Parser<I, Output = O> + 'static,
        R: Parser<I, Output = O> + 'static,
    {
        OrParser::new(vec![Box::new(self), Box::new(right)])
    }

    #[cfg(debug_assertions)]
    fn logging(self, tag: &str) -> crate::pc::LoggingPC<Self>
    where
        Self: Sized,
    {
        crate::pc::LoggingPC::new(self, tag.to_owned())
    }

    fn allow_none_if(self, condition: bool) -> AllowNoneIfParser<Self>
    where
        Self: Sized,
    {
        AllowNoneIfParser::new(self, condition)
    }

    fn zero_or_more(self) -> impl Parser<I, Output = Vec<Self::Output>>
    where
        Self: Sized,
    {
        self.one_or_more().or_default()
    }

    fn one_or_more(self) -> impl Parser<I, Output = Vec<Self::Output>>
    where
        Self: Sized,
    {
        OneOrMoreParser::new(self)
    }

    fn chain<RF, R, F, O>(self, right_factory: RF, combiner: F) -> ChainParser<Self, RF, F>
    where
        Self: Sized,
        RF: Fn(&Self::Output) -> R,
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        ChainParser::new(self, right_factory, combiner)
    }

    fn surround<L, R>(self, left: L, right: R) -> SurroundParser<L, Self, R>
    where
        Self: Sized,
        L: Parser<I>,
        L::Output: Undo,
        R: Parser<I>,
    {
        SurroundParser::new(left, self, right)
    }
}
