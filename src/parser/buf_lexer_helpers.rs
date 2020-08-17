use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;

use std::io::BufRead;

// parser combinators

/// Creates a parser that consumes the next lexeme if it is any word.
pub fn take_if_any_word<I>() -> impl Fn(&mut I) -> Option<Result<Locatable<String>, I::Err>>
where
    I: ResultIterator<Item = LexemeNode> + Transactional,
{
    take_if_map(|lexeme_node| {
        let Locatable { element, pos } = lexeme_node;
        match element {
            Lexeme::Word(word) => Some(word.at(pos)),
            _ => None,
        }
    })
}

/// Creates a parser that consumes the next lexeme if it is any symbol.
pub fn take_if_any_symbol<I>() -> impl Fn(&mut I) -> Option<Result<Locatable<char>, I::Err>>
where
    I: ResultIterator<Item = LexemeNode> + Transactional,
{
    take_if_map(|lexeme_node| {
        let Locatable { element, pos } = lexeme_node;
        match element {
            Lexeme::Symbol(ch) => Some(ch.at(pos)),
            _ => None,
        }
    })
}

/// Creates a parser that consumes the next lexeme if it is the given symbol.
pub fn take_if_symbol<I>(needle: char) -> impl Fn(&mut I) -> Option<Result<Locatable<char>, I::Err>>
where
    I: ResultIterator<Item = LexemeNode> + Transactional,
{
    take_if_map(move |lexeme_node| {
        let Locatable { element, pos } = lexeme_node;
        match element {
            Lexeme::Symbol(ch) => {
                if ch == needle {
                    Some(ch.at(pos))
                } else {
                    None
                }
            }
            _ => None,
        }
    })
}

/// Creates a parser that consumes the next lexeme if it is the given keyword.
pub fn take_if_keyword<I>(
    needle: Keyword,
) -> impl Fn(&mut I) -> Option<Result<Locatable<Keyword>, I::Err>>
where
    I: ResultIterator<Item = LexemeNode> + Transactional,
{
    take_if_map(move |lexeme_node| {
        let Locatable { element, pos } = lexeme_node;
        match element {
            Lexeme::Keyword(k, _) => {
                if k == needle {
                    Some(k.at(pos))
                } else {
                    None
                }
            }
            _ => None,
        }
    })
}

/// Creates a parser that demands that the next lexeme is present.
/// If not, it will return a `SyntaxError` with the given message.
/// This parser therefore never returns `None`.
pub fn demand<I, T, U, FPC, S: AsRef<str>>(err_msg: S, parser: FPC) -> impl Fn(&mut I) -> OptRes<U>
where
    I: ResultIterator<Item = T, Err = QErrorNode> + HasLocation,
    FPC: Fn(&mut I) -> OptRes<U>,
{
    move |source| {
        let pos = source.pos();
        match parser(source) {
            Some(x) => Some(x),
            None => Some(Err(QError::SyntaxError(err_msg.as_ref().to_string())).with_err_at(pos)),
        }
    }
}

/// Creates a parser that parses between two delimiting characters.
///
/// Returns:
///
/// - A tuple with the location of the opening parenthesis and the parsed item
/// - None, if it cannot find the opening symbol
/// - Err, if it cannot find the closing symbol
pub fn between<I, T, FPC>(
    start: char,
    stop: char,
    parser: FPC,
) -> Box<dyn Fn(&mut I) -> OptRes<(Location, T)>>
where
    I: ResultIterator<Item = LexemeNode, Err = QErrorNode> + Transactional + HasLocation + 'static,
    T: 'static,
    FPC: Fn(&mut I) -> OptRes<T> + 'static,
{
    Box::new(apply(
        |(l, r)| (l.pos(), r.0),
        and(
            take_if_symbol(start),
            and(
                parser,
                demand(
                    format!("Unterminated, expected {}", stop),
                    take_if_symbol(stop),
                ),
            ),
        ),
    ))
}

/// Creates a parser that skips the optional leading whitespace before using
/// the given parser to return a result.
pub fn skipping_whitespace<I, T, FPC>(parser: FPC) -> impl Fn(&mut I) -> OptRes<T>
where
    I: ResultIterator<Item = LexemeNode, Err = QErrorNode> + Transactional + HasLocation,
    FPC: Fn(&mut I) -> OptRes<T>,
{
    apply(
        |(_, r)| r,
        zip_allow_left_none(take_if_predicate(LexemeTrait::is_whitespace), parser),
    )
}

/// Creates a parser that demands leading whitespace before using
/// the given parser to return a result.
pub fn demanding_whitespace<I, T, FPC, S: AsRef<str> + 'static>(
    err_msg: S,
    parser: FPC,
) -> Box<dyn Fn(&mut I) -> OptRes<T>>
where
    I: ResultIterator<Item = LexemeNode, Err = QErrorNode> + Transactional + HasLocation + 'static,
    FPC: Fn(&mut I) -> OptRes<T> + 'static,
    T: 'static,
{
    Box::new(apply(
        |(_, r)| r,
        and(
            demand(err_msg, take_if_predicate(LexemeTrait::is_whitespace)),
            parser,
        ),
    ))
}

/// Creates a parser that consumes a list of comma separated values.
/// The values are parsed by the given parser.
/// Whitespace between the elements and the commas is ignored.
/// Trailing comma leads to error.
pub fn csv<I, T, FPC>(item_parser: FPC) -> impl Fn(&mut I) -> OptRes<Vec<T>>
where
    I: ResultIterator<Item = LexemeNode, Err = QErrorNode> + Transactional + HasLocation,
    FPC: Fn(&mut I) -> OptRes<T>,
{
    let item_comma_parser = zip_allow_right_none(
        skipping_whitespace(item_parser),
        skipping_whitespace(take_if_symbol(',')),
    );
    move |source| {
        let mut result: Vec<T> = vec![];
        loop {
            match item_comma_parser(source) {
                None => break,
                Some(Err(err)) => return Some(Err(err)),
                Some(Ok((item, opt_comma))) => {
                    result.push(item);
                    if opt_comma.is_none() {
                        break;
                    }
                }
            }
        }
        Some(Ok(result))
    }
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
