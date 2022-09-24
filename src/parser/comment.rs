use crate::common::*;
use crate::parser::base::*;
use crate::parser::specific::*;
use crate::parser::statement_separator::Separator;
use crate::parser::types::*;

/// Tries to read a comment.
pub fn comment_p() -> impl Parser<Output = Statement> {
    CommentAsString.fn_map(Statement::Comment)
}

/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p() -> impl NonOptParser<Output = Vec<Locatable<String>>> {
    CommentsAndWhitespace.preceded_by_opt_ws()
}

struct CommentsAndWhitespace;

impl HasOutput for CommentsAndWhitespace {
    type Output = Vec<Locatable<String>>;
}

impl NonOptParser for CommentsAndWhitespace {
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<Locatable<String>> = vec![];
        let sep = Separator::Comment;
        let parser = CommentAsString.with_pos();
        let mut found_separator = true;
        let mut found_comment = true;
        while found_separator || found_comment {
            found_separator = sep.parse(tokenizer)?.is_some();
            match parser.parse(tokenizer)? {
                Some(comment) => {
                    result.push(comment);
                    found_comment = true;
                }
                _ => {
                    found_comment = false;
                }
            }
        }
        Ok(result)
    }
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
