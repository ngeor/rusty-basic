// parses DIM statement

use crate::common::*;
use crate::lexer::*;
use crate::parser::error::*;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    if !lexer.peek()?.as_ref().is_symbol('\'') {
        return Ok(None);
    }
    let pos = lexer.read()?.pos();
    let mut buf = String::new();
    while !lexer.peek()?.as_ref().is_eol_or_eof() {
        // TODO move this to a method in LexemeNode e.g. lexeme.push_to_str
        match lexer.read()?.as_ref() {
            Lexeme::Keyword(_, s) | Lexeme::Word(s) | Lexeme::Whitespace(s) => buf.push_str(&s),
            Lexeme::Symbol(c) => {
                buf.push(*c);
            }
            Lexeme::Digits(d) => buf.push_str(&format!("{}", d)),
            Lexeme::EOF | Lexeme::EOL(_) => panic!("should not come here"),
        }
    }
    Ok(Statement::Comment(buf).at(pos)).map(|x| Some(x))
}

#[cfg(test)]
mod tests {}
