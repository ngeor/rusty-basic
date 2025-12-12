use crate::pc::*;
use crate::pc_specific::*;
use crate::types::*;

/// Tries to read a comment.
pub fn comment_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    comment_as_string_p().map(Statement::Comment)
}

pub fn comment_as_string_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = String> {
    any_token_of(TokenType::SingleQuote)
        .and(
            any_token()
                .filter(|t| !TokenType::Eol.matches(t))
                .zero_or_more(),
        )
        .keep_right()
        .map(token_list_to_string)
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
