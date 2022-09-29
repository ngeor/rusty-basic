use crate::common::QError;
use crate::parser::pc::and_opt::AndOptPC;
use crate::parser::pc::and_opt_factory::AndOptFactoryPC;
use crate::parser::pc::many::ManyParser;
use crate::parser::pc::mappers::{FnMapper, KeepLeftMapper, KeepMiddleMapper, KeepRightMapper};
use crate::parser::pc::{
    Alt2, AndDemandLookingBack, AndPC, AndThen, GuardPC, LoggingPC, LoopWhile, LoopWhileNonOpt,
    Seq2, Tokenizer, Undo, ValidateParser,
};

pub trait ParserBase {
    type Output;

    fn and_demand<R>(self, right: R) -> Seq2<Self, R>
    where
        Self: Sized,
        R: NonOptParser,
    {
        Seq2::new(self, right)
    }

    fn and_opt<R>(self, right: R) -> AndOptPC<Self, R>
    where
        Self: Sized,
        R: OptParser,
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

    fn keep_left<L, R>(self) -> KeepLeftMapper<Self>
    where
        Self: Sized + ParserBase<Output = (L, R)>,
    {
        KeepLeftMapper::new(self)
    }

    fn keep_middle<L, M, R>(self) -> KeepMiddleMapper<Self>
    where
        Self: Sized + ParserBase<Output = ((L, M), R)>,
    {
        KeepMiddleMapper::new(self)
    }

    fn keep_right<L, R>(self) -> KeepRightMapper<Self>
    where
        Self: Sized + ParserBase<Output = (L, R)>,
    {
        KeepRightMapper::new(self)
    }

    fn or<O, R>(self, right: R) -> Alt2<O, Self, R>
    where
        Self: Sized + ParserBase<Output = O>,
        R: ParserBase<Output = O>,
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
}

/// A parser that either succeeds or returns an error.
pub trait NonOptParser: ParserBase {
    // TODO it is possible to have a default implementation for `parse` based on `parse_non_opt` if we have a QError that means "no match"
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError>;

    fn loop_while_non_opt<F>(self, f: F) -> LoopWhileNonOpt<Self, F>
    where
        Self: Sized,
    {
        LoopWhileNonOpt::new(self, f)
    }
}

// TODO rename to OptParser
/// A parser that either succeeds, or returns nothing, or returns an error.
pub trait OptParser: ParserBase {
    // note that because of the mut impl parameter, Rust can't convert this trait into
    // an object
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError>;

    fn and<R>(self, right: R) -> AndPC<Self, R>
    where
        Self: Sized,
    {
        AndPC::new(self, right)
    }

    fn and_demand_looking_back<F>(self, factory: F) -> AndDemandLookingBack<Self, F>
    where
        Self: Sized,
    {
        AndDemandLookingBack::new(self, factory)
    }

    fn and_opt_factory<F, R>(self, f: F) -> AndOptFactoryPC<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> R,
        R: OptParser,
    {
        AndOptFactoryPC::new(self, f)
    }

    fn then_use<R>(self, other: R) -> GuardPC<Self, R>
    where
        Self: Sized,
        R: NonOptParser,
    {
        GuardPC::new(self, other)
    }

    fn validate<F>(self, f: F) -> ValidateParser<Self, F>
    where
        Self: Sized,
        Self::Output: Undo,
        F: Fn(&Self::Output) -> Result<bool, QError>,
    {
        ValidateParser::new(self, f)
    }

    fn zero_or_more(self) -> ManyParser<Self>
    where
        Self: Sized,
    {
        ManyParser::new(self, true)
    }
    fn one_or_more(self) -> ManyParser<Self>
    where
        Self: Sized,
    {
        ManyParser::new(self, false)
    }
}
