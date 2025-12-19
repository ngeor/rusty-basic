use crate::pc::many::OneOrMoreParser;
use crate::pc::{
    AllowNoneIfParser, ChainParser, LoopWhile, MapOkNone, MessageProvider, NoIncompleteParser,
    OrFailParser, OrParser, ParseResult, SurroundParser, Tokenizer, Undo, WithExpectedMessage,
};
use crate::ParseError;

// TODO make QError generic param too

/// A parser uses a [Tokenizer] in order to produce a result.
pub trait Parser<I: Tokenizer + 'static> {
    type Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError>;

    /**
     * Not reviewed yet
     */

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
