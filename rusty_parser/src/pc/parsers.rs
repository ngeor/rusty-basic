use crate::pc::and_opt::AndOptPC;
use crate::pc::many::OneOrMoreParser;
use crate::pc::mappers::{FnMapper, KeepLeftMapper, KeepRightMapper};
use crate::pc::{
    AllowDefaultParser, AllowNoneIfParser, AllowNoneParser, AndPC, AndThen, AndThenOkErr,
    ChainParser, FilterMapParser, FilterParser, GuardPC, LoopWhile, MapIncompleteErrParser,
    NoIncompleteParser, OrFailParser, OrParser, ParseResult, SurroundParser, Tokenizer, Undo,
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

    // TODO #[deprecated]
    fn and_opt<R>(self, right: R) -> AndOptPC<Self, R>
    where
        Self: Sized,
    {
        AndOptPC::new(self, right)
    }

    fn and_then<F, U>(self, mapper: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> ParseResult<U, ParseError>,
    {
        AndThen::new(self, mapper)
    }

    /// Flat map the result of this parser for successful and incomplete results.
    /// Other errors are never allowed to be re-mapped.
    fn and_then_ok_err<F, G, U>(
        self,
        ok_mapper: F,
        incomplete_mapper: G,
    ) -> AndThenOkErr<Self, F, G>
    where
        Self: Sized,
        F: Fn(Self::Output) -> Result<U, ParseError>,
        G: Fn(ParseError) -> Result<U, ParseError>,
    {
        AndThenOkErr::new(self, ok_mapper, incomplete_mapper)
    }

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

    fn map<F, U>(self, mapper: F) -> FnMapper<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U,
    {
        FnMapper::new(self, mapper)
    }

    fn map_incomplete_err(self, err: ParseError) -> MapIncompleteErrParser<Self>
    where
        Self: Sized,
    {
        MapIncompleteErrParser::new(self, err)
    }

    fn or_fail(self, err: ParseError) -> OrFailParser<Self>
    where
        Self: Sized,
    {
        OrFailParser::new(self, err)
    }

    fn no_incomplete(self) -> NoIncompleteParser<Self>
    where
        Self: Sized,
    {
        NoIncompleteParser::new(self)
    }

    fn keep_left<L, R>(self) -> KeepLeftMapper<Self>
    where
        Self: Sized + Parser<I, Output = (L, R)>,
    {
        KeepLeftMapper::new(self)
    }

    fn keep_right<L, R>(self) -> KeepRightMapper<Self>
    where
        Self: Sized + Parser<I, Output = (L, R)>,
    {
        KeepRightMapper::new(self)
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

    #[deprecated]
    fn parse_opt(&self, tokenizer: &mut I) -> ParseResult<Option<Self::Output>, ParseError> {
        match self.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(Some(value)),
            ParseResult::None
            | ParseResult::Err(ParseError::Incomplete)
            | ParseResult::Err(ParseError::Expected(_)) => ParseResult::Ok(None),
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }

    fn allow_none(self) -> AllowNoneParser<Self>
    where
        Self: Sized,
    {
        AllowNoneParser::new(self)
    }

    fn allow_none_if(self, condition: bool) -> AllowNoneIfParser<Self>
    where
        Self: Sized,
    {
        AllowNoneIfParser::new(self, condition)
    }

    fn allow_default(self) -> AllowDefaultParser<Self>
    where
        Self: Sized,
        Self::Output: Default,
    {
        AllowDefaultParser::new(self)
    }

    fn and<R>(self, right: R) -> AndPC<Self, R>
    where
        Self: Sized,
        Self::Output: Undo,
        R: Parser<I>,
    {
        AndPC::new(self, right)
    }

    fn then_demand<R>(self, other: R) -> GuardPC<Self, R>
    where
        Self: Sized,
        R: Parser<I>,
    {
        GuardPC::new(self, other)
    }

    fn zero_or_more(self) -> AllowDefaultParser<OneOrMoreParser<Self>>
    where
        Self: Sized,
    {
        self.one_or_more().allow_default()
    }

    fn one_or_more(self) -> OneOrMoreParser<Self>
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
