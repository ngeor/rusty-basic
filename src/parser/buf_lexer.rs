use crate::common::*;
use crate::lexer::*;
use crate::parser::error::*;
use std::io::BufRead;

/// Demands that the given function can parse the next lexeme(s).
/// If the function returns None, it will be converted to an unexpected parser error.
pub fn demand<T: BufRead, TResult, F, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    mut op: F,
    msg: S,
) -> Result<TResult, ParserErrorNode>
where
    F: FnMut(&mut BufLexer<T>) -> Result<Option<TResult>, ParserErrorNode>,
{
    let p = lexer.peek()?;
    match op(lexer) {
        Ok(opt) => match opt {
            Some(x) => Ok(x),
            None => unexpected(msg, p),
        },
        Err(e) => Err(e),
    }
}

pub fn demand_skipping_whitespace<T: BufRead, TResult, F, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    op: F,
    msg: S,
) -> Result<TResult, ParserErrorNode>
where
    F: FnMut(&mut BufLexer<T>) -> Result<Option<TResult>, ParserErrorNode>,
{
    skip_whitespace(lexer)?;
    demand(lexer, op, msg)
}

pub fn in_transaction<T: BufRead, F, TResult>(
    lexer: &mut BufLexer<T>,
    mut op: F,
) -> Result<Option<TResult>, ParserErrorNode>
where
    F: FnMut(&mut BufLexer<T>) -> Result<TResult, ParserErrorNode>,
{
    lexer.begin_transaction();
    match op(lexer) {
        Ok(s) => {
            lexer.commit_transaction();
            Ok(Some(s))
        }
        Err(err) => {
            lexer.rollback_transaction();
            match err.as_ref() {
                ParserError::Unexpected(_, _) => Ok(None),
                _ => Err(err),
            }
        }
    }
}

pub fn skip_if<T: BufRead, F>(lexer: &mut BufLexer<T>, f: F) -> Result<bool, LexerErrorNode>
where
    F: Fn(&Lexeme) -> bool,
{
    let next = lexer.peek()?;
    if f(next.as_ref()) {
        lexer.read()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn read_keyword<T: BufRead>(
    lexer: &mut BufLexer<T>,
    keyword: Keyword,
) -> Result<Location, ParserErrorNode> {
    let Locatable { element: x, pos } = lexer.read()?;
    if x.is_keyword(keyword) {
        Ok(pos)
    } else {
        unexpected(format!("Expected keyword {}", keyword), x.at(pos))
    }
}

pub fn read_whitespace<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<(), ParserErrorNode> {
    let x = lexer.read()?;
    if x.as_ref().is_whitespace() {
        Ok(())
    } else {
        unexpected(format!("Expected whitespace"), x)
    }
}

pub fn read_symbol<T: BufRead>(
    lexer: &mut BufLexer<T>,
    symbol: char,
) -> Result<(), ParserErrorNode> {
    let x = lexer.read()?;
    if x.as_ref().is_symbol(symbol) {
        Ok(())
    } else {
        unexpected(format!("Expected symbol {}", symbol), x)
    }
}

// whitespace

pub fn skip_whitespace<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<(), ParserErrorNode> {
    while lexer.peek()?.as_ref().is_whitespace() {
        lexer.read()?;
    }
    Ok(())
}

pub fn read_demand_whitespace<T: BufRead, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    msg: S,
) -> Result<(), ParserErrorNode> {
    let next = lexer.read()?;
    match next.as_ref() {
        Lexeme::Whitespace(_) => Ok(()),
        _ => unexpected(msg, next),
    }
}

pub fn read_preserve_whitespace<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<(Option<LexemeNode>, LexemeNode), ParserErrorNode> {
    let first = lexer.read()?;
    if first.as_ref().is_whitespace() {
        Ok((Some(first), lexer.read()?))
    } else {
        Ok((None, first))
    }
}

// symbol

pub fn read_demand_symbol_skipping_whitespace<T: BufRead>(
    lexer: &mut BufLexer<T>,
    ch: char,
) -> Result<(), ParserErrorNode> {
    skip_whitespace(lexer)?;
    let next = lexer.read()?;
    if next.as_ref().is_symbol(ch) {
        Ok(())
    } else {
        unexpected(format!("Expected {}", ch), next)
    }
}

// keyword

pub fn read_demand_keyword<T: BufRead>(
    lexer: &mut BufLexer<T>,
    keyword: Keyword,
) -> Result<(), ParserErrorNode> {
    let next = lexer.read()?;
    if next.as_ref().is_keyword(keyword) {
        Ok(())
    } else {
        unexpected(format!("Expected keyword {}", keyword), next)
    }
}
