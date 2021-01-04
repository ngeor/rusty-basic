use crate::common::*;
use crate::parser::pc::binary::BinaryParser;
use crate::parser::pc::many::ManyParser;
use crate::parser::pc::text::{eol_or_whitespace_p, string_while_p};
use crate::parser::pc::unary::UnaryParser;
use crate::parser::pc::unary_fn::UnaryFnParser;
use crate::parser::pc::{is_eol, item_p, Parser, Reader};
use crate::parser::types::*;

fn is_not_eol(ch: char) -> bool {
    !is_eol(ch)
}

/// Tries to read a comment.
pub fn comment_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    item_p('\'')
        .and_opt(string_while_p(is_not_eol))
        .keep_right()
        .map(|x| Statement::Comment(x.unwrap_or_default()))
}

/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p<R>() -> impl Parser<R, Output = Vec<Locatable<String>>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    eol_or_whitespace_p()
        .map_none_to_default()
        .and_opt(
            item_p('\'')
                .with_pos()
                .and_opt(string_while_p(is_not_eol))
                .and_opt(eol_or_whitespace_p())
                .keep_left()
                .map(|(Locatable { pos, .. }, opt_s)| opt_s.unwrap_or_default().at(pos))
                .one_or_more(),
        )
        .and_opt(eol_or_whitespace_p())
        .keep_middle()
        .map(|x| x.unwrap_or_default())
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
