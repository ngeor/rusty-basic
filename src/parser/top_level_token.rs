// top level token ::=
//      comment |
//      def type |
//      declaration |
//      statement |
//      function implementation |
//      sub implementation |
//      whitespace - empty line

use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::declaration;
use crate::parser::def_type;
use crate::parser::error::*;
use crate::parser::implementation;
use crate::parser::statement;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<TopLevelTokenNode>, ParserErrorNode> {
    def_type::try_read(lexer)
        .or_try_read(|| declaration::try_read(lexer))
        .or_try_read(|| implementation::try_read(lexer))
        .or_try_read(|| try_read_statement(lexer))
}

fn try_read_statement<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<TopLevelTokenNode>, ParserErrorNode> {
    statement::try_read(lexer).map(to_top_level_opt)
}

fn to_top_level_opt(x: Option<StatementNode>) -> Option<TopLevelTokenNode> {
    x.map(to_top_level)
}

fn to_top_level(x: StatementNode) -> TopLevelTokenNode {
    x.map(|s| TopLevelToken::Statement(s))
}

pub fn parse_top_level_tokens<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<ProgramNode, ParserErrorNode> {
    let mut read_separator = true; // we are the beginning of the file
    let mut tokens: ProgramNode = vec![];

    // allowed to start with space, eol, : (e.g. WHILE A < 5:), ' for comment
    loop {
        let Locatable { element: p, pos } = lexer.peek()?;
        if p.is_eof() {
            return Ok(tokens);
        } else if p.is_whitespace() {
            lexer.read()?;
        } else if p.is_eol() {
            // now we're allowed to read a statement other than comments,
            // and we're in multi-line mode
            lexer.read()?;
            read_separator = true;
        } else if p.is_symbol('\'') {
            // read comment
            // TODO add unit test where comment reads EOF
            let t = demand(lexer, try_read, "Expected comment")?;
            tokens.push(t);
        // Comments do not need an inline separator but they require a EOL/EOF post-separator
        } else if p.is_symbol(':') {
            // TODO is this allowed at the start of a line?
            // single-line statement separator (e.g. WHILE A < 5:A=A+1:WEND)
            lexer.read()?;
            read_separator = true;
        } else {
            // must be a statement
            if read_separator {
                let t = demand(lexer, try_read, "Expected top level token")?;
                tokens.push(t);
                read_separator = false; // reset to ensure we have a separator for the next statement
            } else {
                return Err(ParserError::Unterminated(p)).with_err_at(pos);
            }
        }
    }
}
