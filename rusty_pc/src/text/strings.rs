use std::marker::PhantomData;

use crate::many::{IgnoringManyCombiner, ManyCombiner, StringManyCombiner};
use crate::text::{any_char, one_char};
use crate::{InputTrait, Parser, ParserErrorTrait};

/// Parses one specific character and returns it as a string.
pub fn one_char_to_str<I, E>(ch: char) -> impl Parser<I, Output = String, Error = E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    one_char(ch).map(String::from)
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
    any_char().filter(predicate).many(combiner)
}

/// Parses the specific string, case insensitive.
pub fn specific_str<I, E>(needle: String) -> impl Parser<I, Output = String, Error = E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    SpecificString::new(needle, StringManyCombiner)
}

/// Parses the specific string, case insensitive, ignoring the output.
pub fn specific_str_ignoring<I, E>(needle: String) -> impl Parser<I, Output = (), Error = E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    SpecificString::new(needle, IgnoringManyCombiner)
}

/// Wraps a [String] so that it can be used as a specific parser.
pub struct SpecificString<O, E, A> {
    needle: String,
    combiner: A,
    _marker: PhantomData<(O, E)>,
}

impl<O, E, A> SpecificString<O, E, A> {
    pub fn new(needle: String, combiner: A) -> Self {
        Self {
            needle,
            combiner,
            _marker: PhantomData,
        }
    }
}

impl<I, O, E, A> Parser<I> for SpecificString<O, E, A>
where
    I: InputTrait<Output = char>,
    O: Default,
    E: ParserErrorTrait,
    A: ManyCombiner<char, O>,
{
    type Output = O;
    type Error = E;

    fn parse(&mut self, input: &mut I) -> Result<O, E> {
        parse_specific_str(&self.needle, &self.combiner, input)
    }
}

fn parse_specific_str<I, O, E>(
    needle: &str,
    combiner: &impl ManyCombiner<char, O>,
    input: &mut I,
) -> Result<O, E>
where
    I: InputTrait<Output = char>,
    O: Default,
    E: ParserErrorTrait,
{
    let mut buffer = O::default();
    let mut success = true;
    // the input's index when we start reading
    let offset = input.get_position();
    {
        for expected_ch in needle.chars() {
            // ensure we're not at eof
            if !input.is_eof() {
                // check the actual character at the input without advancing the input's position
                let actual_ch = input.read();
                if actual_ch.eq_ignore_ascii_case(&expected_ch) {
                    // it's a match, add it and increment the number of successfully read characters
                    buffer = combiner.accumulate(buffer, expected_ch);
                } else {
                    success = false;
                    break;
                }
            } else {
                success = false;
                break;
            }
        }
    }

    if success {
        Ok(buffer)
    } else {
        input.set_position(offset);
        Err(E::default()) // TODO: ParserError::expected(needle))
    }
}
