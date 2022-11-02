use crate::pc::and_opt::AndOptPC;
use crate::pc::many::OneOrMoreParser;
use crate::pc::mappers::{FnMapper, KeepLeftMapper, KeepRightMapper};
use crate::pc::{
    AllowDefaultParser, AllowNoneIfParser, AllowNoneParser, Alt2, AndPC, AndThen, ChainParser,
    FilterMapParser, FilterParser, GuardPC, LoopWhile, MapIncompleteErrParser, NegateParser,
    NoIncompleteParser, OrFailParser, ParserToParserOnceAdapter, PeekParser, Tokenizer, Undo,
};
use rusty_common::*;

// TODO make QError generic param too
// TODO specific error types for Tokenizer and Parser libraries

/// A parser uses a [Tokenizer] in order to produce a result.
///
/// There are two different types of failures:
/// - incomplete: another parser might be able to succeed
/// - fatal: all parsing should stop
///
/// The [Tokenizer] is available through a `mut impl` parameter on the method
/// [Parser::parse]. This choice has some pros and cons. The `impl Tokenizer`
/// means that this is in reality a generic method, meaning that the Parser
/// can't be converted into a `dyn object`. On the positive side, the type
/// is simpler, with no extra generic parameters polluting the definitions.
pub trait Parser {
    type Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError>;

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
        F: Fn(Self::Output) -> Result<U, QError>,
    {
        AndThen::new(self, mapper)
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

    fn map_incomplete_err(self, err: QError) -> MapIncompleteErrParser<Self>
    where
        Self: Sized,
    {
        MapIncompleteErrParser::new(self, err)
    }

    fn or_fail(self, err: QError) -> OrFailParser<Self>
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
        Self: Sized + Parser<Output = (L, R)>,
    {
        KeepLeftMapper::new(self)
    }

    fn keep_right<L, R>(self) -> KeepRightMapper<Self>
    where
        Self: Sized + Parser<Output = (L, R)>,
    {
        KeepRightMapper::new(self)
    }

    fn or<O, R>(self, right: R) -> Alt2<O, Self, R>
    where
        Self: Sized + Parser<Output = O>,
        R: Parser<Output = O>,
    {
        Alt2::new(self, right)
    }

    #[cfg(debug_assertions)]
    fn logging(self, tag: &str) -> crate::pc::LoggingPC<Self>
    where
        Self: Sized,
    {
        crate::pc::LoggingPC::new(self, tag.to_owned())
    }

    // TODO #[deprecated]
    fn parse_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parse(tokenizer) {
            Ok(value) => Ok(Some(value)),
            Err(QError::Incomplete) | Err(QError::Expected(_)) => Ok(None),
            Err(err) => Err(err),
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
        R: Parser,
    {
        AndPC::new(self, right)
    }

    fn then_demand<R>(self, other: R) -> GuardPC<Self, R>
    where
        Self: Sized,
        R: Parser + NonOptParser,
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

    fn peek(self) -> PeekParser<Self>
    where
        Self: Sized,
    {
        PeekParser::new(self)
    }

    // TODO #[deprecated]
    fn negate(self) -> NegateParser<Self>
    where
        Self: Sized,
    {
        NegateParser::new(self)
    }

    fn chain<RF, R>(self, right_factory: RF) -> ChainParser<Self, RF>
    where
        Self: Sized,
        RF: Fn(Self::Output) -> R,
        R: ParserOnce,
    {
        ChainParser::new(self, right_factory)
    }

    /// Converts a [Parser] to a [ParserOnce].
    /// A blanket implementation is technically possible,
    /// but creates problems when generic parameters need to be
    /// adjusted e.g. from [Fn] to [FnOnce].
    fn to_parser_once(self) -> ParserToParserOnceAdapter<Self>
    where
        Self: Sized,
    {
        ParserToParserOnceAdapter::new(self)
    }
}

/// A parser that returns a successful result or a fatal error.
/// This parser will never return an error that is "incomplete".
/// TODO: review all direct impl NonOptParser outside the core parsers, as implementing a marker trait doesn't guarantee much
pub trait NonOptParser: Parser {}

// TODO try an OptParser trait which has the conversions to NonOptParser methods such as or_syntax_error
// TODO mimic the std::iter functions to create new parsers from simpler blocks

/// A parser that can only be used once. Similar to `FnOnce`.
pub trait ParserOnce {
    type Output;

    fn parse(self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError>;

    fn map<F, U>(self, mapper: F) -> FnMapper<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> U,
    {
        FnMapper::new(self, mapper)
    }

    fn or<O, R>(self, right: R) -> Alt2<O, Self, R>
    where
        Self: Sized + ParserOnce<Output = O>,
        R: ParserOnce<Output = O>,
    {
        Alt2::new(self, right)
    }
}

// TODO remove all "impl Parser" outside the main framework
