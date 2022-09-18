use crate::common::*;
use crate::parser::base::and_pc::AndTrait;
use crate::parser::base::parsers::{HasOutput, KeepRightTrait, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::specific::{item_p, whitespace, WithPosTrait};
use crate::parser::statement;
use crate::parser::statement_separator::StatementSeparator;
use crate::parser::types::*;

pub fn single_line_non_comment_statements_p() -> impl Parser<Output = StatementNodes> {
    whitespace()
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

pub fn single_line_statements_p() -> impl Parser<Output = StatementNodes> {
    whitespace()
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
pub fn zero_or_more_statements_p<S>(exit_source: S) -> impl Parser<Output = StatementNodes>
where
    S: Parser,
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

struct StatementAndSeparator;

impl StatementAndSeparator {
    pub fn new() -> Self {
        Self
    }
}

impl HasOutput for StatementAndSeparator {
    type Output = StatementNode;
}

impl Parser for StatementAndSeparator {
    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_statement_node = statement::statement_p().with_pos().parse(reader)?;
        match opt_statement_node {
            Some(statement_node) => {
                let is_comment = if let Statement::Comment(_) = statement_node.as_ref() {
                    true
                } else {
                    false
                };
                let opt_separator = StatementSeparator::new(is_comment).parse(reader)?;
                match opt_separator {
                    Some(_) => Ok(Some(statement_node)),
                    _ => Err(QError::syntax_error("Expected: end-of-statement")),
                }
            }
            _ => Ok(None),
        }
    }
}
