use crate::many::{ManyCombiner, StringManyCombiner};
use crate::{InputTrait, Parser, ParserErrorTrait, one_p, read_p};

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
