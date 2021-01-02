use crate::common::*;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::str::zero_or_more_if;
use crate::parser::pc::ws::{is_eol, is_eol_or_whitespace};
use crate::parser::pc::*;
use crate::parser::pc_specific::with_pos;
use crate::parser::types::*;

fn is_not_eol(ch: char) -> bool {
    !is_eol(ch)
}

/// Tries to read a comment.
pub fn comment<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Statement, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map(and(read('\''), zero_or_more_if(is_not_eol)), |(_, r)| {
        Statement::Comment(r)
    })
}

/// Reads multiple comments
pub fn comments<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Vec<Locatable<String>>, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    // skip while ws or eol
    // if "'", undo and read comment
    // repeat

    drop_left(and(
        // leading whitespace / eol
        zero_or_more_if(is_eol_or_whitespace),
        many(with_pos(drop_right(and(
            drop_left(and(read('\''), zero_or_more_if(is_not_eol))),
            zero_or_more_if(is_eol_or_whitespace),
        )))),
    ))
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
