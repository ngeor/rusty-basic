use crate::common::*;
use crate::parser::base::delimited_pc::DelimitedTrait;
use crate::parser::base::parsers::{HasOutput, NonOptParser, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::base::undo_pc::Undo;
use crate::parser::specific::item_p;
use crate::parser::specific::whitespace::WhitespaceTrait;
use crate::parser::specific::with_pos::WithPosTrait;
use crate::parser::statement;
use crate::parser::statement_separator::Separator;
use crate::parser::types::*;

pub fn single_line_non_comment_statements_p() -> impl Parser<Output = StatementNodes> {
    statement::single_line_non_comment_statement_p()
        .with_pos()
        .one_or_more_delimited_by(
            item_p(':').surrounded_by_opt_ws(),
            QError::syntax_error("Error: trailing colon"),
        )
        .preceded_by_req_ws()
}

pub fn single_line_statements_p() -> impl Parser<Output = StatementNodes> {
    statement::single_line_statement_p()
        .with_pos()
        .one_or_more_delimited_by(
            item_p(':').surrounded_by_opt_ws(),
            QError::syntax_error("Error: trailing colon"),
        )
        .preceded_by_req_ws()
}

pub struct ZeroOrMoreStatements<S>(NegateParser<S>, Option<QError>);

impl<S> ZeroOrMoreStatements<S> {
    pub fn new(exit_source: S) -> Self {
        Self(NegateParser(exit_source), None)
    }

    pub fn new_with_custom_error(exit_source: S, err: QError) -> Self {
        Self(NegateParser(exit_source), Some(err))
    }
}

impl<S> HasOutput for ZeroOrMoreStatements<S> {
    type Output = StatementNodes;
}

impl<S> NonOptParser for ZeroOrMoreStatements<S>
where
    S: Parser,
    S::Output: Undo,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        // must start with a separator (e.g. after a WHILE condition)
        Separator::NonComment
            .parse(tokenizer)?
            .ok_or(QError::syntax_error("Expected: end-of-statement"))?;
        let mut result: StatementNodes = vec![];
        let mut state = 0;
        // while not found exit
        while self.0.parse(tokenizer)?.is_some() {
            if state == 0 || state == 2 {
                // looking for statement
                if let Some(statement_node) =
                    statement::statement_p().with_pos().parse(tokenizer)?
                {
                    result.push(statement_node);
                    state = 1;
                } else {
                    return Err(match &self.1 {
                        Some(custom_error) => custom_error.clone(),
                        _ => QError::syntax_error("Expected: statement"),
                    });
                }
            } else if state == 1 {
                // looking for separator after statement
                let found_separator =
                    if let Some(Statement::Comment(_)) = result.last().map(|x| &x.element) {
                        // last element was comment
                        Separator::Comment.parse(tokenizer)?.is_some()
                    } else {
                        Separator::NonComment.parse(tokenizer)?.is_some()
                    };
                if found_separator {
                    state = 2;
                } else {
                    return Err(QError::syntax_error("Expected: statement separator"));
                }
            } else {
                panic!("Cannot happen")
            }
        }
        Ok(result)
    }
}

struct NegateParser<P>(P);

impl<P> HasOutput for NegateParser<P> {
    type Output = ();
}

impl<P> Parser for NegateParser<P>
where
    P: Parser,
    P::Output: Undo,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => {
                value.undo(tokenizer);
                Ok(None)
            }
            None => Ok(Some(())),
        }
    }
}
