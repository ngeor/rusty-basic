use crate::common::*;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statement;
use crate::parser::statement_separator::Separator;
use crate::parser::types::*;

pub fn single_line_non_comment_statements_p() -> impl Parser<Output = StatementNodes> {
    whitespace()
        .and(delimited_by_colon(
            statement::single_line_non_comment_statement_p().with_pos(),
        ))
        .keep_right()
}

pub fn single_line_statements_p() -> impl Parser<Output = StatementNodes> {
    whitespace()
        .and(delimited_by_colon(
            statement::single_line_statement_p().with_pos(),
        ))
        .keep_right()
}

fn delimited_by_colon<P: Parser>(parser: P) -> impl Parser<Output = Vec<P::Output>> {
    delimited_by(
        parser,
        colon_ws(),
        QError::syntax_error("Error: trailing colon"),
    )
}

pub struct ZeroOrMoreStatements<S>(NegateParser<PeekParser<S>>, Option<QError>);

impl<S> ZeroOrMoreStatements<S>
where
    S: Parser,
    S::Output: Undo,
{
    pub fn new(exit_source: S) -> Self {
        Self(exit_source.peek().negate(), None)
    }

    pub fn new_with_custom_error(exit_source: S, err: QError) -> Self {
        Self(exit_source.peek().negate(), Some(err))
    }
}

impl<S> Parser for ZeroOrMoreStatements<S>
where
    S: Parser,
    S::Output: Undo,
{
    type Output = StatementNodes;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        // must start with a separator (e.g. after a WHILE condition)
        Separator::NonComment
            .parse_opt(tokenizer)?
            .ok_or_else(|| QError::syntax_error("Expected: end-of-statement"))?;
        let mut result: StatementNodes = vec![];
        // TODO rewrite the numeric state or add constants
        let mut state = 0;
        // while not found exit
        while self.0.parse_opt(tokenizer)?.is_some() {
            if state == 0 || state == 2 {
                // looking for statement
                if let Some(statement_node) =
                    statement::statement_p().with_pos().parse_opt(tokenizer)?
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
                        Separator::Comment.parse_opt(tokenizer)?.is_some()
                    } else {
                        Separator::NonComment.parse_opt(tokenizer)?.is_some()
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

// TODO review impl<...> NonOptParser
impl<S> NonOptParser for ZeroOrMoreStatements<S>
where
    S: Parser,
    S::Output: Undo,
{
}
