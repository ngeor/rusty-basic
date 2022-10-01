use crate::common::*;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;

/// Tries to read a comment.
pub fn comment_p() -> impl Parser<Output = Statement> {
    CommentAsString.map(Statement::Comment)
}

pub struct CommentAsString;

impl Parser for CommentAsString {
    type Output = String;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
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
                Ok(result)
            }
            Some(token) => {
                tokenizer.unread(token);
                Err(QError::Incomplete)
            }
            None => Err(QError::Incomplete),
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
        let program = parse_str(input);
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
        let program = parse_str(input);
        assert_eq!(
            program,
            vec![TopLevelToken::Statement(Statement::Comment(String::new())).at_rc(1, 1)]
        );
    }
}
