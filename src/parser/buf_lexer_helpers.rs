use crate::common::*;
use crate::lexer::*;

use std::io::BufRead;

// parser combinators

/// Creates a parser that gets the next lexeme if it is a word.
pub fn take_if_any_word<I>() -> impl Fn(&mut I) -> Option<Result<Locatable<String>, I::Err>>
where
    I: PeekResultIterator<Item = LexemeNode>,
{
    take_if(LexemeTrait::is_word, |lexeme_node| {
        let Locatable { element, pos } = lexeme_node;
        match element {
            Lexeme::Word(word) => Some(word.at(pos)),
            _ => None,
        }
    })
}

/// Creates a parser that gets the next symbol if it is a symbol.
pub fn take_if_any_symbol_parser<I>() -> impl Fn(&mut I) -> Option<Result<Locatable<char>, I::Err>>
where
    I: PeekResultIterator<Item = LexemeNode>,
{
    take_if(
        |lexeme_node| {
            let Locatable { element, .. } = lexeme_node;
            match element {
                Lexeme::Symbol(_) => true,
                _ => false,
            }
        },
        |lexeme_node| {
            let Locatable { element, pos } = lexeme_node;
            match element {
                Lexeme::Symbol(ch) => Some(ch.at(pos)),
                _ => None,
            }
        },
    )
}

/// Demands that the given function can parse the next lexeme(s).
/// If the function returns None, it will be converted to a syntax error.
///
/// # Parameters
///
/// - `lexer`: The buffering lexer
/// - `parse_function`: A function that will be called to parse the next lexeme. If the function
///   returns None, it will be converted to a syntax error.
/// - `msg`: The error message for the syntax error.
#[deprecated]
pub fn read<T: BufRead, TResult, F, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    mut parse_function: F,
    msg: S,
) -> Result<TResult, QErrorNode>
where
    F: FnMut(&mut BufLexer<T>) -> Result<Option<TResult>, QErrorNode>,
{
    let p = lexer.peek_ref_ng()?;
    match p {
        Some(x) => {
            let pos = x.pos();
            match parse_function(lexer) {
                Ok(opt) => match opt {
                    Some(x) => Ok(x),
                    None => Err(QError::SyntaxError(msg.as_ref().to_string())).with_err_at(pos),
                },
                Err(e) => Err(e),
            }
        }
        None => {
            let pos = lexer.pos();
            Err(QError::SyntaxError(msg.as_ref().to_string())).with_err_at(pos)
        }
    }
}

/// Runs the given function in a transaction. Syntax errors will be
/// converted to a `Ok(None)`.
///
/// This allows a (sub)parser to go ahead until it encounters a syntax error
/// and backtrack to the past known location.
///
/// # Parameters
///
/// - lexer: The buffering lexer used to get lexemes
/// - parse_function: The parsing function
#[deprecated]
pub fn in_transaction<T: BufRead, F, TResult>(
    lexer: &mut BufLexer<T>,
    mut parse_function: F,
) -> Result<Option<TResult>, QErrorNode>
where
    F: FnMut(&mut BufLexer<T>) -> Result<TResult, QErrorNode>,
{
    lexer.begin_transaction();
    match parse_function(lexer) {
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

#[deprecated]
pub fn skip_if<T: BufRead, F>(lexer: &mut BufLexer<T>, f: F) -> Result<bool, QErrorNode>
where
    F: Fn(&Lexeme) -> bool,
{
    match lexer.peek_ref_ng()? {
        Some(next) => {
            if f(next.as_ref()) {
                lexer.read_ng()?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
        None => Ok(false),
    }
}

#[deprecated]
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

#[deprecated]
pub fn read_symbol<T: BufRead>(lexer: &mut BufLexer<T>, symbol: char) -> Result<(), QErrorNode> {
    let x = lexer.read()?;
    if x.as_ref().is_symbol(symbol) {
        Ok(())
    } else {
        Err(QError::SyntaxError(format!("Expected symbol {}", symbol))).with_err_at(&x)
    }
}

/// Reads lexemes as long as they are whitespace.
/// Returns `true` if at least one whitespace was read.
#[deprecated]
pub fn skip_whitespace<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<bool, QErrorNode> {
    let mut found_whitespace = false;
    while lexer.peek_ref_ng().is_whitespace() {
        lexer.read_ng()?;
        found_whitespace = true;
    }
    Ok(found_whitespace)
}

#[deprecated]
pub fn read_whitespace<T: BufRead, S: AsRef<str>>(
    lexer: &mut BufLexer<T>,
    msg: S,
) -> Result<(), QErrorNode> {
    match lexer.read_ng()? {
        Some(Locatable {
            element: Lexeme::Whitespace(_),
            ..
        }) => Ok(()),
        Some(Locatable { pos, .. }) => {
            Err(QError::SyntaxError(msg.as_ref().to_string())).with_err_at(pos)
        }
        _ => Err(QError::SyntaxError(msg.as_ref().to_string())).with_err_at(lexer.pos()),
    }
}
