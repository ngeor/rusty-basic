use super::{BareName, BareNameNode, Keyword, Name, NameNode};
use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::map::source_and_then_some;
use crate::parser::pc::*;
use crate::parser::pc2::{any_identifier_with_dot_p, Parser};
use crate::parser::pc_specific::*;
use crate::parser::type_qualifier::type_qualifier_p;
use crate::parser::{type_qualifier, TypeQualifier};
use std::io::BufRead;
use std::str::FromStr;

// name node

pub fn name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, NameNode, QError>> {
    with_pos(name())
}

#[deprecated]
pub fn name<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Name, QError>> {
    name_with_dot_p().convert_to_fn()
}

pub fn name_with_dot_p<R>() -> impl Parser<R, Output = Name>
where
    R: Reader<Item = char, Err = QError> + 'static,
{
    any_identifier_with_dot_p()
        .validate(|n| {
            if n.len() > MAX_LENGTH {
                Err(QError::IdentifierTooLong)
            } else {
                Ok(true)
            }
        })
        .and_opt(type_qualifier_p())
        .map(|(n, opt_q)| Name::new(n.into(), opt_q))
        .validate(|n| {
            let bare_name: &BareName = n.as_ref();
            let s: &str = bare_name.as_ref();
            let is_keyword = Keyword::from_str(s).is_ok();
            if is_keyword {
                match n.qualifier() {
                    Some(TypeQualifier::DollarString) => Ok(true),
                    Some(_) => Err(QError::syntax_error("Unexpected keyword")),
                    _ => {
                        // undo everything
                        Ok(false)
                    }
                }
            } else {
                Ok(true)
            }
        })
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
