use crate::common::*;
use crate::parser::base::parsers::Parser;
use crate::parser::specific::item_p;
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

fn is_not_eol(ch: char) -> bool {
    !is_eol(ch)
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
