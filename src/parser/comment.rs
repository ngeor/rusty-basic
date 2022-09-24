use crate::common::*;
use crate::parser::base::delimited_pc::DelimitedTrait;
use crate::parser::base::parsers::{FnMapTrait, HasOutput, NonOptParser, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::specific::whitespace::WhitespaceTrait;
use crate::parser::specific::with_pos::WithPosTrait;
use crate::parser::specific::TokenType;
use crate::parser::statement_separator::Separator;
use crate::parser::types::*;

/// Tries to read a comment.
pub fn comment_p() -> impl Parser<Output = Statement> {
    CommentAsString.fn_map(Statement::Comment)
}

/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p() -> impl NonOptParser<Output = Vec<Locatable<String>>> {
    CommentAsString
        .with_pos()
        .one_or_more_delimited_by_allow_missing(Separator::Comment)
        .fn_map(keep_non_empty)
        .preceded_by_opt_ws()
}

fn keep_non_empty<E>(x: Vec<Option<E>>) -> Vec<E> {
    x.into_iter()
        .filter(Option::is_some)
        .map(Option::unwrap)
        .collect()
}

struct CommentAsString;

impl HasOutput for CommentAsString {
    type Output = String;
}

impl Parser for CommentAsString {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) if token.kind == TokenType::SingleQuote as i32 => {
                let mut result = String::new();
                while let Some(token) = tokenizer.read()? {
                    if token.kind == TokenType::Eol as i32 {
                        tokenizer.unread(token);
                        break;
                    } else {
                        result.push_str(&token.text);
                    }
                }
                Ok(Some(result))
            }
            Some(token) => {
                tokenizer.unread(token);
                Ok(None)
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;

    #[test]
    fn test_comment_until_eof() {
        let input = "' just a comment . 123 AS";
        let program = parse(input);
        assert_eq!(
            program,
            vec![TopLevelToken::Statement(Statement::Comment(
                " just a comment . 123 AS".to_string()
            ))
            .at_rc(1, 1)]
        );
    }

    #[test]
    fn test_comment_at_eof() {
        let input = "'";
        let program = parse(input);
        assert_eq!(
            program,
            vec![TopLevelToken::Statement(Statement::Comment(String::new())).at_rc(1, 1)]
        );
    }
}
