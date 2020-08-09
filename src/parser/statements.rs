use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;

use crate::parser::statement;
use crate::parser::types::*;
use std::io::BufRead;

#[derive(Debug, Default)]
pub struct ParseStatementsOptions {
    pub first_statement_separated_by_whitespace: bool,
}

pub fn parse_statements<T: BufRead, F, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    exit_predicate: F,
    err_msg: S,
) -> Result<StatementNodes, QErrorNode>
where
    F: Fn(&Lexeme) -> bool,
{
    parse_statements_with_options(
        lexer,
        exit_predicate,
        err_msg,
        ParseStatementsOptions::default(),
    )
}

pub fn parse_statements_with_options<T: BufRead, F, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    exit_predicate: F,
    err_msg: S,
    options: ParseStatementsOptions,
) -> Result<StatementNodes, QErrorNode>
where
    F: Fn(&Lexeme) -> bool,
{
    let mut read_separator = false;
    let mut statements: StatementNodes = vec![];

    // allowed to start with space, eol, : (e.g. WHILE A < 5:), ' for comment
    loop {
        let Locatable { element: p, pos } = lexer.peek()?;
        if exit_predicate(&p) {
            // found the exit door
            // important that this check is done first, e.g. in case EOL or EOF is part of the exit predicate
            return Ok(statements);
        } else if p.is_eof() {
            return Err(QError::SyntaxError(err_msg.as_ref().to_string())).with_err_at(pos);
        } else if p.is_whitespace() {
            lexer.read()?;
            if statements.is_empty() && options.first_statement_separated_by_whitespace {
                read_separator = true;
            }
        } else if p.is_eol() {
            // now we're allowed to read a statement other than comments,
            // and we're in multi-line mode
            lexer.read()?;
            read_separator = true;
        } else if p.is_symbol('\'') {
            // read comment, regardless of whether we've seen the separator or not
            // TODO add unit test where comment reads EOF
            let s = demand(lexer, statement::try_read, "Expected comment")?;
            statements.push(s);
        // Comments do not need an inline separator but they require a EOL/EOF post-separator
        } else if p.is_symbol(':') {
            // single-line statement separator (e.g. WHILE A < 5:A=A+1:WEND)
            lexer.read()?;
            read_separator = true;
        } else {
            // must be a statement
            if read_separator {
                let s = demand(lexer, statement::try_read, "Expected statement")?;
                statements.push(s);
                read_separator = false; // reset to ensure we have a separator for the next statement
            } else {
                return Err(QError::SyntaxError(format!(
                    "Expected statement: {}",
                    err_msg.as_ref()
                )))
                .with_err_at(pos);
            }
        }
    }
}
