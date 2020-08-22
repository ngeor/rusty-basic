use crate::char_reader::*;
use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;

use crate::parser::buf_lexer_helpers::*;
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

#[deprecated]
pub fn take_if_comment<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    apply(
        |(left, lexeme_nodes)| {
            let pos = left.pos();
            let text =
                lexeme_nodes
                    .into_iter()
                    .fold(String::new(), |acc, Locatable { element, .. }| {
                        format!("{}{}", acc, element) // concatenate strings
                    });
            Statement::Comment(text).at(pos)
        },
        and(take_if_symbol('\''), take_until(LexemeTrait::is_eol)),
    )
}

#[deprecated]
pub fn try_read<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNode>, QErrorNode> {
    take_if_comment()(lexer).transpose()
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
