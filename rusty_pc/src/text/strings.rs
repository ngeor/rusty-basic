use crate::many::{ManyCombiner, StringManyCombiner};
use crate::{
    IgnoringParser, IgnoringPredicate, InputTrait, Parser, ParserErrorTrait, one_p, read_p
};

/// Parses one specific character and returns it as a string.
pub fn one_char_to_str<I, E>(ch: char) -> impl Parser<I, Output = String, Error = E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    one_p(ch).map(String::from)
}

/// Parses one or more characters that match the given predicate
/// and returns a String.
pub fn many_str<I, E, F>(predicate: F) -> impl Parser<I, Output = String, Error = E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
    F: Fn(&char) -> bool,
{
    many_str_with_combiner(predicate, StringManyCombiner)
}

/// Parses one or more characters that match the given predicate,
/// collecting them with the given combiner.
pub fn many_str_with_combiner<I, O, E, F, C>(
    predicate: F,
    combiner: C,
) -> impl Parser<I, Output = O, Error = E>
where
    I: InputTrait<Output = char>,
    O: Default, // needed because `many` can support allowing no results
    E: ParserErrorTrait,
    F: Fn(&char) -> bool,
    C: ManyCombiner<char, O>,
{
    read_p().filter(predicate).many(combiner)
}

/// A predicate that is used in order to skip whitespace.
/// The whitespace is defined as spaces or tabs.
pub struct WhitespaceIgnoringPredicate;

impl IgnoringPredicate for WhitespaceIgnoringPredicate {
    type Output = char;

    fn test(&self, element: Self::Output) -> bool {
        element == ' ' || element == '\t'
    }
}

/// Creates a parser that skips while whitespace is encountered.
/// At least one whitespace character must be found for the parser to succeed.
pub fn whitespace_ignoring<E>() -> IgnoringParser<WhitespaceIgnoringPredicate, E>
where
    E: ParserErrorTrait,
{
    WhitespaceIgnoringPredicate.parser(false)
}

/// Creates a parser that skips while whitespace is encountered.
/// The parser will only fail on EOF, as zero whitespace characters is also acceptable.
pub fn opt_ws<E>() -> IgnoringParser<WhitespaceIgnoringPredicate, E>
where
    E: ParserErrorTrait,
{
    WhitespaceIgnoringPredicate.parser(true)
}
