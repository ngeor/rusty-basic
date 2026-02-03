use crate::text::one_char;
use crate::{InputTrait, Parser, ParserErrorTrait};

/// Parses one specific character and returns it as a string.
pub fn one<I, E>(ch: char) -> impl Parser<I, Output = String, Error = E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    one_char(ch).map(String::from)
}
