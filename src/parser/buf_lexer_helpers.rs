use crate::common::*;
use crate::lexer::*;

use std::io::BufRead;

/// Demands that the given function can parse the next lexeme(s).
/// If the function returns None, it will be converted to a syntax error.
pub fn demand<T: BufRead, TResult, F, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    mut op: F,
    msg: S,
) -> Result<TResult, QErrorNode>
where
    F: FnMut(&mut BufLexer<T>) -> Result<Option<TResult>, QErrorNode>,
{
    let p = lexer.peek()?;
    match op(lexer) {
        Ok(opt) => match opt {
            Some(x) => Ok(x),
            None => Err(QError::SyntaxError(msg.as_ref().to_string())).with_err_at(&p),
        },
        Err(e) => Err(e),
    }
}

pub fn demand_skipping_whitespace<T: BufRead, TResult, F, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    op: F,
    msg: S,
) -> Result<TResult, QErrorNode>
where
    F: FnMut(&mut BufLexer<T>) -> Result<Option<TResult>, QErrorNode>,
{
    skip_whitespace(lexer)?;
    demand(lexer, op, msg)
}

pub fn in_transaction<T: BufRead, F, TResult>(
    lexer: &mut BufLexer<T>,
    mut op: F,
) -> Result<Option<TResult>, QErrorNode>
where
    F: FnMut(&mut BufLexer<T>) -> Result<TResult, QErrorNode>,
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
                QError::SyntaxError(_) => Ok(None),
                _ => Err(err),
            }
        }
    }
}

pub fn skip_if<T: BufRead, F>(lexer: &mut BufLexer<T>, f: F) -> Result<bool, QErrorNode>
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
) -> Result<Location, QErrorNode> {
    let Locatable { element: x, pos } = lexer.read()?;
    if x.is_keyword(keyword) {
        Ok(pos)
    } else {
        Err(QError::SyntaxError(format!("Expected keyword {}", keyword))).with_err_at(pos)
    }
}

pub fn read_whitespace<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<(), QErrorNode> {
    let x = lexer.read()?;
    if x.as_ref().is_whitespace() {
        Ok(())
    } else {
        Err(QError::SyntaxError(format!("Expected whitespace"))).with_err_at(&x)
    }
}

pub fn read_symbol<T: BufRead>(lexer: &mut BufLexer<T>, symbol: char) -> Result<(), QErrorNode> {
    let x = lexer.read()?;
    if x.as_ref().is_symbol(symbol) {
        Ok(())
    } else {
        Err(QError::SyntaxError(format!("Expected symbol {}", symbol))).with_err_at(&x)
    }
}

// whitespace

pub fn skip_whitespace<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<(), QErrorNode> {
    while lexer.peek()?.as_ref().is_whitespace() {
        lexer.read()?;
    }
    Ok(())
}

pub fn read_demand_whitespace<T: BufRead, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    msg: S,
) -> Result<(), QErrorNode> {
    let next = lexer.read()?;
    match next.as_ref() {
        Lexeme::Whitespace(_) => Ok(()),
        _ => Err(QError::SyntaxError(msg.as_ref().to_string())).with_err_at(&next),
    }
}

pub fn read_preserve_whitespace<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<(Option<LexemeNode>, LexemeNode), QErrorNode> {
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
) -> Result<(), QErrorNode> {
    skip_whitespace(lexer)?;
    let next = lexer.read()?;
    if next.as_ref().is_symbol(ch) {
        Ok(())
    } else {
        Err(QError::SyntaxError(format!("Expected symbol {}", ch))).with_err_at(&next)
    }
}

// keyword

pub fn read_demand_keyword<T: BufRead>(
    lexer: &mut BufLexer<T>,
    keyword: Keyword,
) -> Result<(), QErrorNode> {
    let next = lexer.read()?;
    if next.as_ref().is_keyword(keyword) {
        Ok(())
    } else {
        Err(QError::SyntaxError(format!("Expected keyword {}", keyword))).with_err_at(&next)
    }
}
