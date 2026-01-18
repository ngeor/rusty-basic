use rusty_pc::*;

use crate::input::StringView;
use crate::tokens::*;
use crate::{ParserError, *};

/// Parses a comment as a statement.
/// Does not consume the EOL token.
pub fn comment_p() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    comment_as_string_p().map(Statement::Comment)
}

/// Parses a comment as a [String].
/// Does not consume the EOL token.
pub fn comment_as_string_p() -> impl Parser<StringView, Output = String, Error = ParserError> {
    // TODO support ignoring tokens to avoid allocation
    any_symbol_of!('\'').and_keep_right(
        any_token_of!(TokenType::Eol ; mode = MatchMode::Exclude)
            .many_allow_none(StringManyCombiner),
    )
}

#[cfg(test)]
mod tests {
    use rusty_common::AtPos;

    use crate::*;

    #[test]
    fn test_comment_until_eof() {
        let input = "' just a comment . 123 AS";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::Comment(
                    " just a comment . 123 AS".to_string()
                ))
                .at_rc(1, 1)
            ]
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
