use crate::pc::and::{And, AndWithoutUndo};
use crate::pc::flat_map::{FlatMap, FlatMapOkNoneClosures};
use crate::pc::many::OneOrMoreParser;
use crate::pc::map::{Map, MapOkNone, MapOkNoneTrait, MapToDefault, MapToOption};
use crate::pc::{
    AllowNoneIfParser, ChainParser, FilterMapParser, FilterParser, LoopWhile, MessageProvider,
    NoIncompleteParser, OrFailParser, OrParser, ParseResult, SurroundParser, Tokenizer, Undo,
    WithExpectedMessage,
};
use crate::ParseError;

// TODO make QError generic param too

/// A parser uses a [Tokenizer] in order to produce a result.
pub trait Parser<I: Tokenizer + 'static> {
    type Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError>;

    /**
     * And
     */

    /// Parses both the left and the right side.
    /// If the right side fails, parsing of the left side is undone.
    fn and<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        And::new(self, right, combiner)
    }

    fn and_tuple<R>(self, right: R) -> impl Parser<I, Output = (Self::Output, R::Output)>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
    {
        self.and(right, |l, r| (l, r))
    }

    fn and_keep_left<R>(self, right: R) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
    {
        self.and(right, |l, _| l)
    }

    fn and_keep_right<R>(self, right: R) -> impl Parser<I, Output = R::Output>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
    {
        self.and(right, |_, r| r)
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
        AndWithoutUndo::new(self, right, combiner)
    }

    /// Parses the left side and returns the right side.
    /// If the left does not succeed, the right is not parsed.
    /// Be careful: If the right does not succeed, the left is not undone.
    /// This should not be used unless it's certain that the right can't fail.
    /// TODO use a NonOptParser here for the right side.
    fn and_without_undo_keep_right<R>(self, right: R) -> impl Parser<I, Output = R::Output>
    where
        Self: Sized,
        R: Parser<I>,
    {
        self.and_without_undo(right, |_, right| right)
    }

    /// Parses the left side and optionally the right side.
    /// The combiner function maps the left and (optional) right output to the final result.
    fn and_opt<R, F, O>(self, right: R, combiner: F) -> impl Parser<I, Output = O>
    where
        Self: Sized,
        R: Parser<I> + 'static,
        F: Fn(Self::Output, Option<R::Output>) -> O,
    {
        self.and_without_undo(right.to_option(), combiner)
    }

    /// Parses the left side and optionally the right side.
    /// The result is a tuple of both sides.
    fn and_opt_tuple<R>(
        self,
        right: R,
    ) -> impl Parser<I, Output = (Self::Output, Option<R::Output>)>
    where
        Self: Sized,
        R: Parser<I> + 'static,
    {
        self.and_opt(right, |l, r| (l, r))
    }

    /// Parses the left side and optionally the right side.
    /// The result is only the left side's output.
    fn and_opt_keep_left<R>(self, right: R) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
        R: Parser<I> + 'static,
    {
        self.and_opt(right, |l, _| l)
    }

    /// Parses the left side and optionally the right side.
    /// The result is only the right side's output.
    fn and_opt_keep_right<R>(self, right: R) -> impl Parser<I, Output = Option<R::Output>>
    where
        Self: Sized,
        R: Parser<I> + 'static,
    {
        self.and_opt(right, |_, r| r)
    }

    /**
     * Map
     */

    fn map<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U,
    {
        Map::new(self, mapper)
    }

    #[deprecated]
    fn keep_right<L, R>(self) -> impl Parser<I, Output = R>
    where
        Self: Sized + Parser<I, Output = (L, R)>,
    {
        self.map(|(_, r)| r)
    }

    /// Map the result of this parser for successful and incomplete results.
    /// The given mapper implements [MapOkNoneTrait] which takes care of the mapping.
    fn map_ok_none<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized + 'static,
        F: MapOkNoneTrait<Self::Output, U> + 'static,
    {
        MapOkNone::new(self, mapper)
    }

    fn to_option(self) -> impl Parser<I, Output = Option<Self::Output>>
    where
        Self: Sized + 'static,
    {
        self.map_ok_none(MapToOption)
    }

    fn or_default(self) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized + 'static,
        Self::Output: Default,
    {
        self.map_ok_none(MapToDefault)
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
        FlatMap::new(self, mapper)
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
        FlatMapOkNoneClosures::new(self, ok_mapper, incomplete_mapper)
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
        Self: Sized + 'static,
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
