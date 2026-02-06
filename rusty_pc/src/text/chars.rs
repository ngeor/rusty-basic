use crate::{InputTrait, Parser, ParserErrorTrait, read_p};

/// Parses one specific character.
pub fn one_char<I, E>(ch: char) -> impl Parser<I, Output = char, Error = E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    read_p().filter(move |c: &char| *c == ch)
}
