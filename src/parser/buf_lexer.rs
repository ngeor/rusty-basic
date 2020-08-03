use crate::common::*;
use crate::lexer::*;
use crate::parser::error::*;
use crate::parser::types::*;
use std::io::BufRead;

/// Demands that the given function can parse the next lexeme(s).
/// If the function returns None, it will be converted to an unexpected parser error.
pub fn demand<T: BufRead, TResult, F, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    mut op: F,
    msg: S,
) -> Result<TResult, ParserError>
where
    F: FnMut(&mut BufLexer<T>) -> Result<Option<TResult>, ParserError>,
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
) -> Result<TResult, ParserError>
where
    F: FnMut(&mut BufLexer<T>) -> Result<Option<TResult>, ParserError>,
{
    skip_whitespace(lexer)?;
    demand(lexer, op, msg)
}

pub fn in_transaction<T: BufRead, F, TResult>(
    lexer: &mut BufLexer<T>,
    mut op: F,
) -> Result<Option<TResult>, ParserError>
where
    F: FnMut(&mut BufLexer<T>) -> Result<TResult, ParserError>,
{
    lexer.begin_transaction();
    match op(lexer) {
        Ok(s) => {
            lexer.commit_transaction()?;
            Ok(Some(s))
        }
        Err(err) => {
            lexer.rollback_transaction()?;
            match &err {
                ParserError::Unexpected(_, _) => Ok(None),
                _ => Err(err),
            }
        }
    }
}

pub fn skip_if<T: BufRead, F>(lexer: &mut BufLexer<T>, f: F) -> Result<bool, LexerError>
where
    F: Fn(&LexemeNode) -> bool,
{
    let next = lexer.peek()?;
    if f(&next) {
        lexer.read()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn read_keyword<T: BufRead>(
    lexer: &mut BufLexer<T>,
    keyword: Keyword,
) -> Result<Location, ParserError> {
    let x = lexer.read()?;
    if x.is_keyword(keyword) {
        Ok(x.location())
    } else {
        Err(ParserError::Unexpected(
            format!("Expected keyword {}", keyword),
            x,
        ))
    }
}

pub fn read_whitespace<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<(), ParserError> {
    let x = lexer.read()?;
    if x.is_whitespace() {
        Ok(())
    } else {
        Err(ParserError::Unexpected(format!("Expected whitespace"), x))
    }
}

pub fn read_symbol<T: BufRead>(lexer: &mut BufLexer<T>, symbol: char) -> Result<(), ParserError> {
    let x = lexer.read()?;
    if x.is_symbol(symbol) {
        Ok(())
    } else {
        Err(ParserError::Unexpected(
            format!("Expected symbol {}", symbol),
            x,
        ))
    }
}

pub fn read_bare_name<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<BareName, ParserError> {
    let x = lexer.read()?;
    match x {
        LexemeNode::Word(w, _) => Ok(w.into()),
        _ => Err(ParserError::Unexpected(format!("Expected word"), x)),
    }
}

pub fn read_word_or_keyword<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<BareName, ParserError> {
    let x = lexer.read()?;
    match x {
        LexemeNode::Word(w, _) => Ok(w.into()),
        LexemeNode::Keyword(_, w, _) => Ok(w.into()),
        _ => Err(ParserError::Unexpected(
            format!("Expected word or keyword"),
            x,
        )),
    }
}

// peek but do not read
pub fn peek_keyword<T: BufRead>(
    lexer: &mut BufLexer<T>,
    keyword: Keyword,
) -> Result<bool, ParserError> {
    let x = lexer.peek()?;
    Ok(x.is_keyword(keyword))
}

pub fn peek_symbol<T: BufRead>(lexer: &mut BufLexer<T>, symbol: char) -> Result<bool, ParserError> {
    let x = lexer.peek()?;
    Ok(x.is_symbol(symbol))
}

// whitespace

pub fn skip_whitespace<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<(), ParserError> {
    while lexer.peek()?.is_whitespace() {
        lexer.read()?;
    }
    Ok(())
}

pub fn read_demand_whitespace<T: BufRead, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    msg: S,
) -> Result<(), ParserError> {
    let next = lexer.read()?;
    match next {
        LexemeNode::Whitespace(_, _) => Ok(()),
        _ => unexpected(msg, next),
    }
}

pub fn read_preserve_whitespace<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<(Option<LexemeNode>, LexemeNode), ParserError> {
    let first = lexer.read()?;
    if first.is_whitespace() {
        Ok((Some(first), lexer.read()?))
    } else {
        Ok((None, first))
    }
}

// symbol

pub fn read_demand_symbol_skipping_whitespace<T: BufRead>(
    lexer: &mut BufLexer<T>,
    ch: char,
) -> Result<(), ParserError> {
    skip_whitespace(lexer)?;
    let next = lexer.read()?;
    if next.is_symbol(ch) {
        Ok(())
    } else {
        unexpected(format!("Expected {}", ch), next)
    }
}

// keyword

pub fn read_demand_keyword<T: BufRead>(
    lexer: &mut BufLexer<T>,
    keyword: Keyword,
) -> Result<(), ParserError> {
    let next = lexer.read()?;
    if next.is_keyword(keyword) {
        Ok(())
    } else {
        unexpected(format!("Expected keyword {}", keyword), next)
    }
}
