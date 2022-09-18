use std::str::FromStr;

use crate::common::QError;
use crate::parser::base::parsers::Parser;
use crate::parser::base::tokenizers::{token_list_to_string, Token, Tokenizer};
use crate::parser::specific::TokenType;
use crate::parser::type_qualifier::type_qualifier_p;
use crate::parser::{BareName, Keyword, Name, TypeQualifier};

/// Parses a name. The name must start with a letter and can include
/// letters, digits or dots. The name can optionally be qualified by a type
/// qualifier.
///
/// The parser validates the maximum length of the name and checks that the name
/// is not a keyword (with the exception of strings, e.g. `end$`).
pub fn name_with_dot_p() -> impl Parser<Output = Name> {
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
            let bare_name: &BareName = n.bare_name();
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

pub fn bare_name_p() -> impl Parser<Output = BareName> {
    any_word_with_dot_p().unless_followed_by(type_qualifier_p())
}

pub const MAX_LENGTH: usize = 40;

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn any_word_with_dot_p() -> impl Parser<Output = BareName> {
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
    use crate::parser::specific::create_string_tokenizer;

    use super::*;

    #[test]
    fn test_any_word_with_dot() {
        let inputs = ["abc", "abc1", "abc.def"];
        let expected_outputs = ["abc", "abc1", "abc.def"];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_output = expected_outputs[i];
            let eol_reader = create_string_tokenizer(input);
            let mut parser = any_word_with_dot_p();
            let result = parser.parse(eol_reader).expect("Should succeed");
            assert_eq!(result, Some(BareName::from(expected_output)));
        }
    }
}

struct IdentifierWithDotParser;

impl Parser for IdentifierWithDotParser {
    type Output = String; // TODO check if Vec<Token> will work better for undo

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let mut list: Vec<Token> = Vec::new();
        loop {
            match tokenizer.read()? {
                Some(token) => {
                    if token.kind == TokenType::Identifier as i32 {
                        list.push(token);
                    } else if token.kind == TokenType::Digits as i32 {
                        if list.is_empty() {
                            tokenizer.unread(token);
                            break;
                        } else if list.last().unwrap().kind == TokenType::Dot as i32 {
                            return Err(QError::syntax_error(
                                "Property cannot start with a number",
                            ));
                        } else {
                            list.push(token);
                        }
                    } else if token.kind == TokenType::Dot as i32 {
                        if list.is_empty() {
                            tokenizer.unread(token);
                            break;
                        } else if list.last().unwrap().kind == TokenType::Dot as i32 {
                            return Err(QError::syntax_error("Two dots in a row"));
                        } else {
                            list.push(token);
                        }
                    } else {
                        tokenizer.unread(token);
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }
        if list.is_empty() {
            Ok(None)
        } else {
            Ok(Some(token_list_to_string(list)))
        }
    }
}

// TODO rename to _opt
fn identifier_with_dot() -> impl Parser {
    IdentifierWithDotParser
}
