use std::marker::PhantomData;

use rusty_pc::many::ManyCombiner;
use rusty_pc::{InputTrait, Parser};

use crate::ParserError;
use crate::input::StringView;

/// Wraps a [String] so that it can be used as a specific parser.
pub(super) struct SpecificString<A, O> {
    needle: String,
    combiner: A,
    _marker: PhantomData<O>,
}

impl<A, O> SpecificString<A, O> {
    pub fn new(needle: String, combiner: A) -> Self {
        Self {
            needle,
            combiner,
            _marker: PhantomData,
        }
    }
}

impl<A, O> Parser<StringView> for SpecificString<A, O>
where
    A: ManyCombiner<char, O>,
    O: Default,
{
    type Output = O;
    type Error = ParserError;

    fn parse(&mut self, input: &mut StringView) -> Result<O, ParserError> {
        parse_specific_str(&self.needle, &self.combiner, input)
    }
}

fn parse_specific_str<O>(
    needle: &str,
    combiner: &impl ManyCombiner<char, O>,
    input: &mut StringView,
) -> Result<O, ParserError>
where
    O: Default,
{
    let mut buffer = O::default();
    let mut success = true;

    // how many characters have we read successfully so far
    let mut read: usize = 0;

    {
        // the input's index when we start reading
        let offset = input.index();

        for expected_ch in needle.chars() {
            // ensure we're not at eof
            if offset + read < input.len() {
                // check the actual character at the input without advancing the input's position
                let actual_ch = input.char_at(offset + read);
                if actual_ch.eq_ignore_ascii_case(&expected_ch) {
                    // it's a match, add it and increment the number of successfully read characters
                    buffer = combiner.accumulate(buffer, expected_ch);
                    read += 1;
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
        input.inc_position_by(read);
        Ok(buffer)
    } else {
        Err(ParserError::expected(needle))
    }
}
