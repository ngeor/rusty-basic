use std::str::FromStr;

use crate::common::QError;
use crate::parser::base::and_then_pc::AndThenTrait;
use crate::parser::base::logging::LoggingTrait;
use crate::parser::base::parsers::{AndOptTrait, FnMapTrait, HasOutput, Parser};
use crate::parser::base::tokenizers::{token_list_to_string, Token, TokenList, Tokenizer};
use crate::parser::base::undo_pc::{Undo, UndoTrait};
use crate::parser::specific::TokenType;
use crate::parser::type_qualifier::type_qualifier_as_token;
use crate::parser::{BareName, Keyword, Name, TypeQualifier};

/// Parses a name. The name must start with a letter and can include
/// letters, digits or dots. The name can optionally be qualified by a type
/// qualifier.
///
/// The parser validates the maximum length of the name and checks that the name
/// is not a keyword (with the exception of strings, e.g. `end$`).
pub fn name_with_dot_p() -> impl Parser<Output = Name> {
    identifier_with_dot()
        .logging("identifier_with_dot")
        .and_then(ensure_token_list_length)
        .and_opt(type_qualifier_as_token().logging("type_qualifier_as_token"))
        .validate(|(n, opt_q)| {
            // TODO preserve the string and type qualifier for the fn_map step
            let full_name = token_list_to_string(n);
            let is_keyword = Keyword::from_str(&full_name).is_ok();
            if is_keyword {
                match TypeQualifier::from_opt_token(opt_q) {
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
        .fn_map(|(n, opt_q)| {
            Name::new(
                token_list_to_string(&n).into(),
                TypeQualifier::from_opt_token(&opt_q),
            )
        })
}

// bare name node

pub fn bare_name_p() -> impl Parser<Output = BareName> {
    UnlessFollowedBy(identifier_with_dot(), type_qualifier_as_token())
        .validate(ensure_length_and_not_keyword)
        .fn_map(|x| token_list_to_string(&x).into()) // TODO make a parser for simpler .into() cases
}

struct UnlessFollowedBy<L, R>(L, R);

impl<L, R> HasOutput for UnlessFollowedBy<L, R>
where
    L: HasOutput,
{
    type Output = L::Output;
}

impl<L, R> Parser for UnlessFollowedBy<L, R>
where
    L: Parser,
    L::Output: Undo,
    R: Parser,
    R::Output: Undo,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => match self.1.parse(tokenizer)? {
                Some(needle) => {
                    needle.undo(tokenizer);
                    value.undo(tokenizer);
                    Ok(None)
                }
                None => Ok(Some(value)),
            },
            None => Ok(None),
        }
    }
}

pub const MAX_LENGTH: usize = 40;

fn ensure_length_and_not_keyword(list: &TokenList) -> Result<bool, QError> {
    let s = token_list_to_string(list);
    if s.len() > MAX_LENGTH {
        Err(QError::IdentifierTooLong)
    } else {
        match Keyword::from_str(&s) {
            Ok(_) => Ok(false),
            Err(_) => Ok(true),
        }
    }
}

struct IdentifierWithDotParser;

impl HasOutput for IdentifierWithDotParser {
    type Output = Vec<Token>;
}

impl Parser for IdentifierWithDotParser {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let mut list: Vec<Token> = Vec::new();
        loop {
            match tokenizer.read()? {
                Some(token) => {
                    // need keywords too because `end$` is a valid variable name even though `end` isn't
                    if token.kind == TokenType::Identifier as i32
                        || token.kind == TokenType::Keyword as i32
                    {
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
            Ok(Some(list))
        }
    }
}

// TODO rename to _opt
fn identifier_with_dot() -> IdentifierWithDotParser {
    IdentifierWithDotParser
}

fn ensure_token_list_length(list: Vec<Token>) -> Result<Vec<Token>, QError> {
    if token_list_string_length(&list) > MAX_LENGTH {
        Err(QError::IdentifierTooLong)
    } else {
        Ok(list)
    }
}

fn token_list_string_length(list: &[Token]) -> usize {
    let mut result: usize = 0;
    for item in list {
        result += item.text.len();
    }
    result
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
            let mut eol_reader = create_string_tokenizer(input);
            let parser = bare_name_p();
            let result = parser.parse(&mut eol_reader).expect("Should succeed");
            assert_eq!(result, Some(BareName::from(expected_output)));
        }
    }
}
