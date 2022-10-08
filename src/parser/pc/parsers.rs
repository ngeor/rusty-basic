use crate::common::QError;
use crate::parser::pc::and_opt::AndOptPC;
use crate::parser::pc::many::OneOrMoreParser;
use crate::parser::pc::mappers::{FnMapper, KeepLeftMapper, KeepMiddleMapper, KeepRightMapper};
use crate::parser::pc::{
    AllowDefaultParser, AllowNoneIfParser, AllowNoneParser, Alt2, AndPC, AndThen, ChainParser,
    FilterMapParser, FilterParser, GuardPC, LoggingPC, LoopWhile, MapIncompleteErrParser, MapOnce,
    NegateParser, NoIncompleteParser, OrFailParser, PeekParser, Tokenizer, Undo,
};

// TODO V4: the tokenizer is not visible (practically an iterator)
// pub trait ParserV4 {
//     type Output;
//     fn parse(&mut self) -> Result<Self::Output, QError>;
// }
// alternatively, move the Tokenizer impl up as generic parameter

// TODO make QError generic param too after figuring out <T> vs associated type

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

    fn map_once<F, U>(self, mapper: F) -> MapOnce<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> U,
    {
        MapOnce::new(self, mapper)
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

    fn keep_middle<L, M, R>(self) -> KeepMiddleMapper<Self>
    where
        Self: Sized + Parser<Output = ((L, M), R)>,
    {
        KeepMiddleMapper::new(self)
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
    fn logging(self, tag: &str) -> LoggingPC<Self>
    where
        Self: Sized,
    {
        LoggingPC::new(self, tag.to_owned())
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
}

// TODO remove all "impl Parser" outside the main framework
