// parses DIM statement

use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::error::*;
use crate::parser::types::*;
use std::io::BufRead;

// TODO get rid also of the <T: BufRead> from here

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    if peek_symbol(lexer, '\'')? {
        let pos = lexer.read()?.location();
        let mut buf = String::new();
        while !lexer.peek()?.is_eol_or_eof() {
            // TODO move this to a method in LexemeNode e.g. lexeme.push_to_str
            match lexer.read()? {
                LexemeNode::Keyword(_, s, _)
                | LexemeNode::Word(s, _)
                | LexemeNode::Whitespace(s, _) => buf.push_str(&s),
                LexemeNode::Symbol(c, _) => {
                    buf.push(c);
                }
                LexemeNode::Digits(d, _) => buf.push_str(&format!("{}", d)),
                LexemeNode::EOF(_) | LexemeNode::EOL(_, _) => panic!("should not come here"),
            }
        }
        Ok(Statement::Comment(buf).at(pos)).map(|x| Some(x))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {}
