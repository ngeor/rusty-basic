use crate::pc::many::Many;
use crate::pc::{
    AllowNoneIfParser, And, ChainParser, LoopWhile, MessageProvider, NoIncompleteParser, OrDefault,
    OrFailParser, ParseResult, WithExpectedMessage,
};
use crate::ParseError;

// TODO make QError generic param too

/// A parser uses a [Tokenizer] in order to produce a result.
pub trait Parser<I> {
    type Output;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, ParseError>;

    /**
     * Not reviewed yet
     */

    fn loop_while<F>(self, predicate: F) -> LoopWhile<Self, F>
    where
        Self: Sized,
        I: Clone,
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
        I: Clone,
    {
        self.one_or_more().or_default()
    }

    fn one_or_more(self) -> impl Parser<I, Output = Vec<Self::Output>>
    where
        Self: Sized,
        I: Clone,
    {
        Many::new(
            self,
            |e| vec![e],
            |mut v: Vec<Self::Output>, e| {
                v.push(e);
                v
            },
        )
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

    fn surround<L, R>(self, left: L, right: R) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
        I: Clone,
        L: Parser<I>,
        R: Parser<I>,
    {
        left.and_keep_right(self).and_keep_left(right)
    }
}
