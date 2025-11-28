use crate::pc::*;
use crate::pc_specific::*;
use crate::types::*;
use crate::ParseError;

/// Tries to read a comment.
pub fn comment_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    comment_as_string_p().map(Statement::Comment)
}

pub fn comment_as_string_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = String> {
    CommentAsString
}

struct CommentAsString;

impl<I: Tokenizer + 'static> Parser<I> for CommentAsString {
    type Output = String;
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match tokenizer.read()? {
            Some(token) if TokenType::SingleQuote.matches(&token) => {
                let mut result = String::new();
                while let Some(token) = tokenizer.read()? {
                    if TokenType::Eol.matches(&token) {
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
                Err(ParseError::Incomplete)
            }
            None => Err(ParseError::Incomplete),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_comment_until_eof() {
        let input = "' just a comment . 123 AS";
        let program = parse(input);
        assert_eq!(
            program,
            vec![GlobalStatement::Statement(Statement::Comment(
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
            vec![GlobalStatement::Statement(Statement::Comment(String::new())).at_rc(1, 1)]
        );
    }
}
