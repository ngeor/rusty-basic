use crate::common::*;
use crate::parser::pc::*;
use crate::parser::statement;
use crate::parser::statement_separator::StatementSeparator;
use crate::parser::types::*;
use std::marker::PhantomData;

pub fn single_line_non_comment_statements_p<R>() -> impl Parser<R, Output = StatementNodes>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(
            statement::single_line_non_comment_statement_p()
                .with_pos()
                .one_or_more_delimited_by(
                    item_p(':').surrounded_by_opt_ws(),
                    QError::syntax_error("Error: trailing colon"),
                ),
        )
        .keep_right()
}

pub fn single_line_statements_p<R>() -> impl Parser<R, Output = StatementNodes>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(
            statement::single_line_statement_p()
                .with_pos()
                .one_or_more_delimited_by(
                    item_p(':').surrounded_by_opt_ws(),
                    QError::syntax_error("Error: trailing colon"),
                ),
        )
        .keep_right()
}

// When `zero_or_more_statements_p` is called, it must always read first the first statement separator.
// `top_level_token` handles the case where the first statement does not start with
// a separator.
pub fn zero_or_more_statements_p<R, S>(exit_source: S) -> impl Parser<R, Output = StatementNodes>
where
    R: Reader<Item = char, Err = QError> + HasLocation + Undo<S::Output> + 'static,
    S: Parser<R> + 'static,
{
    // first separator
    // loop
    //      negate exit source
    //      statement node and separator
    StatementSeparator::new(false)
        .and(
            exit_source
                .negate()
                .and(StatementAndSeparator::new())
                .keep_right()
                .zero_or_more(),
        )
        .keep_right()
}

struct StatementAndSeparator<R>(PhantomData<R>);

impl<R> StatementAndSeparator<R>
where
    R: Reader<Item = char, Err = QError> + HasLocation,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R> Parser<R> for StatementAndSeparator<R>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    type Output = StatementNode;

    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_statement_node) = statement::statement_p().with_pos().parse(reader)?;
        match opt_statement_node {
            Some(statement_node) => {
                let is_comment = if let Statement::Comment(_) = statement_node.as_ref() {
                    true
                } else {
                    false
                };
                let (reader, opt_separator) = StatementSeparator::new(is_comment).parse(reader)?;
                match opt_separator {
                    Some(_) => Ok((reader, Some(statement_node))),
                    _ => Err((reader, QError::syntax_error("Expected: end-of-statement"))),
                }
            }
            _ => Ok((reader, None)),
        }
    }
}
