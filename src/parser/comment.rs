use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
use crate::parser::pc::str::zero_or_more_if;
use crate::parser::pc::ws::is_eol;
use crate::parser::types::*;
use std::io::BufRead;

fn is_not_eol(ch: char) -> bool {
    !is_eol(ch)
}

/// Tries to read a comment.
pub fn comment<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QError>)> {
    map(
        opt_seq2(try_read('\''), zero_or_more_if(is_not_eol)),
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
