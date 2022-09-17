use crate::common::*;
use crate::parser::base::parsers::{filter_token, filter_token_by_kind_opt, Parser};
use crate::parser::specific::{item_p, TokenType};
use crate::parser::types::*;

/// Tries to read a comment.
pub fn comment_p() -> impl Parser<Output = Statement> {
    item_p('\'')
        .and_opt(non_eol_p())
        .keep_right()
        .map(|x| Statement::Comment(x.unwrap_or_default()))
}

/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p() -> impl Parser<Output = Vec<Locatable<String>>> {
    eol_or_whitespace_p()
        .map_none_to_default()
        .and_opt(
            item_p('\'')
                .with_pos()
                .and_opt(non_eol_p())
                .and_opt(eol_or_whitespace_p())
                .keep_left()
                .map(|(Locatable { pos, .. }, opt_s)| opt_s.unwrap_or_default().at(pos))
                .one_or_more(),
        )
        .and_opt(eol_or_whitespace_p())
        .keep_middle()
        .map(|x| x.unwrap_or_default())
}

// TODO rename to non_eol_opt
fn non_eol_p() -> impl Parser {
    filter_token(|token| Ok(token.kind != TokenType::Eol as i32))
}

fn eol_or_whitespace_p() -> impl Parser {
    filter_token(|token| {
        Ok(token.kind == TokenType::Eol as i32 || token.kind == TokenType::Whitespace as i32)
    })
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
