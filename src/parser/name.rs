use crate::common::{HasLocation, QError};
use crate::parser::pc::{Reader, ReaderResult};
use crate::parser::pc2::binary::BinaryParser;
use crate::parser::pc2::unary_fn::UnaryFnParser;
use crate::parser::pc2::Parser;
use crate::parser::pc_specific::identifier_with_dot;
use crate::parser::type_qualifier::type_qualifier_p;
use crate::parser::{BareName, Keyword, Name, TypeQualifier};
use std::str::FromStr;

/// Parses a name. The name must start with a letter and can include
/// letters, digits or dots. The name can optionally be qualified by a type
/// qualifier.
///
/// The parser validates the maximum length of the name and checks that the name
/// is not a keyword (with the exception of strings, e.g. `end$`).
pub fn name_with_dot_p<R>() -> impl Parser<R, Output = Name>
where
    R: Reader<Item = char, Err = QError> + 'static,
{
    identifier_with_dot()
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

#[deprecated]
pub fn bare_name<R>() -> Box<dyn Fn(R) -> ReaderResult<R, BareName, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    bare_name_p().convert_to_fn()
}

pub fn bare_name_p<R>() -> impl Parser<R, Output = BareName>
where
    R: Reader<Item = char, Err = QError> + 'static,
{
    any_word_with_dot_p().unless_followed_by(type_qualifier_p())
}

pub const MAX_LENGTH: usize = 40;

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn any_word_with_dot_p<R>() -> impl Parser<R, Output = BareName>
where
    R: Reader<Item = char, Err = QError> + 'static,
{
    identifier_with_dot()
        .validate(ensure_length_and_not_keyword)
        .map(|x| x.into())
}

fn ensure_length_and_not_keyword(s: &String) -> Result<bool, QError> {
    if s.len() > MAX_LENGTH {
        Err(QError::IdentifierTooLong)
    } else {
        match Keyword::from_str(s.as_ref()) {
            Ok(_) => Ok(false),
            Err(_) => Ok(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::char_reader::EolReader;

    #[test]
    fn test_any_word_with_dot() {
        let inputs = ["abc", "abc1", "abc.def"];
        let expected_outputs = ["abc", "abc1", "abc.def"];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_output = expected_outputs[i];
            let eol_reader = EolReader::from(input);
            let parser = any_word_with_dot_p();
            let (_, result) = parser.parse(eol_reader).expect("Should succeed");
            assert_eq!(result, Some(BareName::from(expected_output)));
        }
    }
}
