use crate::char_reader::*;
use crate::common::*;
use crate::parser::types::*;
use std::io::BufRead;

/// Tries to read a comment.
pub fn comment<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    map_ng(
        if_first_maybe_second(
            try_read_char('\''),
            read_any_str_while(|ch| ch != '\r' && ch != '\n'),
        ),
        |(_, r)| Statement::Comment(r.unwrap_or_default()),
    )
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
