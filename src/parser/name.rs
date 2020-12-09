use super::{BareName, BareNameNode, Keyword, Name, NameNode};
use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::map::source_and_then_some;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::{type_qualifier, TypeQualifier};
use std::io::BufRead;
use std::str::FromStr;

// name node

pub fn name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, NameNode, QError>> {
    with_pos(name())
}

pub fn name<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Name, QError>> {
    source_and_then_some(
        opt_seq2(any_identifier_with_dot(), type_qualifier::type_qualifier()),
        |reader, (bare_name, opt_q)| {
            if bare_name.len() > MAX_LENGTH {
                Err((reader, QError::IdentifierTooLong))
            } else {
                let is_keyword = Keyword::from_str(bare_name.as_ref()).is_ok();
                if is_keyword {
                    match opt_q {
                        Some(TypeQualifier::DollarString) => {
                            // this is surprisingly allowed
                            Ok((reader, Some(Name::new(bare_name.into(), opt_q))))
                        }
                        Some(_) => Err((reader, QError::syntax_error("Unexpected keyword"))),
                        None => {
                            // let's undo everything
                            Ok((reader.undo(bare_name), None))
                        }
                    }
                } else {
                    Ok((reader, Some(Name::new(bare_name.into(), opt_q))))
                }
            }
        },
    )
}

// bare name node

pub fn bare_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareNameNode, QError>> {
    with_pos(bare_name())
}

pub fn bare_name<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareName, QError>> {
    drop_right(and(
        any_word_with_dot(),
        negate(type_qualifier::type_qualifier()),
    ))
}

pub const MAX_LENGTH: usize = 40;

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn any_word_with_dot<T: BufRead + 'static>(
) -> impl Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareName, QError> {
    source_and_then_some(any_identifier_with_dot(), ensure_length_and_not_keyword)
}

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn any_word_without_dot<T: BufRead + 'static>(
) -> impl Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareName, QError> {
    source_and_then_some(any_identifier_without_dot(), ensure_length_and_not_keyword)
}

fn ensure_length_and_not_keyword<T: BufRead + 'static>(
    reader: EolReader<T>,
    s: String,
) -> ReaderResult<EolReader<T>, BareName, QError> {
    if s.len() > MAX_LENGTH {
        Err((reader, QError::IdentifierTooLong))
    } else {
        match Keyword::from_str(&s) {
            Ok(_) => Ok((reader.undo(s), None)),
            Err(_) => Ok((reader, Some(s.into()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_any_word_without_dot() {
        let inputs = ["abc", "abc1", "abc.def", "a$"];
        let expected_outputs = ["abc", "abc1", "abc", "a"];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_output = expected_outputs[i];
            let eol_reader = EolReader::from(input);
            let parser = any_word_without_dot();
            let (_, result) = parser(eol_reader).expect("Should succeed");
            assert_eq!(result, Some(BareName::from(expected_output)));
        }
    }

    #[test]
    fn test_any_word_with_dot() {
        let inputs = ["abc", "abc1", "abc.def"];
        let expected_outputs = ["abc", "abc1", "abc.def"];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_output = expected_outputs[i];
            let eol_reader = EolReader::from(input);
            let parser = any_word_with_dot();
            let (_, result) = parser(eol_reader).expect("Should succeed");
            assert_eq!(result, Some(BareName::from(expected_output)));
        }
    }
}
