/// Deals with characters and strings.
/// The Reader is always a Reader<Item = char>
use super::{Parser, Reader, ReaderResult, Undo};
use crate::parser::pc::ws::is_whitespace;
use crate::parser::pc2::binary::{LeftAndOptRight, OptLeftAndRight};
use crate::parser::pc2::unary::{MapNoneToDefault, UnaryParser};
use std::marker::PhantomData;

// Find a string

pub struct ReadString<R: Reader<Item = char>> {
    needle: &'static str,
    reader: PhantomData<R>,
}

pub fn read_string_p<R: Reader<Item = char>>(needle: &'static str) -> ReadString<R> {
    ReadString {
        needle,
        reader: PhantomData,
    }
}

impl<R> Parser<R> for ReadString<R>
where
    R: Reader<Item = char>,
{
    type Output = String;
    fn parse(&self, r: R) -> ReaderResult<R, String, R::Err> {
        let mut reader = r;
        let mut result = String::new();
        for n in self.needle.chars() {
            let res = reader.read();
            match res {
                Ok((r, Some(ch))) => {
                    result.push(ch);
                    if ch.to_ascii_uppercase() == n.to_ascii_uppercase() {
                        reader = r;
                    } else {
                        return Ok((r.undo(result), None));
                    }
                }
                Ok((r, None)) => {
                    // EOF before matching all characters, undo collected and return None
                    return Ok((r.undo(result), None));
                }
                Err((r, err)) => {
                    // Error occurred, exit fast
                    return Err((r, err));
                }
            }
        }
        Ok((reader, Some(result)))
    }
}

// Read one or more characters that meet the predicate

pub struct ReadOneOrMoreWhile<R, F>(F, PhantomData<R>);

impl<R, F> ReadOneOrMoreWhile<R, F>
where
    R: Reader<Item = char>,
    F: Fn(char) -> bool,
{
    pub fn new(predicate: F) -> Self {
        Self(predicate, PhantomData)
    }
}

impl<R, F> Parser<R> for ReadOneOrMoreWhile<R, F>
where
    R: Reader<Item = char>,
    F: Fn(char) -> bool,
{
    type Output = String;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        do_parse_while(reader, &self.0)
    }
}

fn do_parse_while<R, F>(reader: R, predicate: F) -> ReaderResult<R, String, R::Err>
where
    R: Reader<Item = char>,
    F: Fn(char) -> bool,
{
    let mut result = String::new();
    let mut r = reader;
    let mut has_more = true;
    while has_more {
        let (tmp, opt_item) = r.read()?;
        r = tmp;
        match opt_item {
            Some(item) => {
                if predicate(item) {
                    result.push(item);
                } else {
                    r = r.undo_item(item);
                    has_more = false;
                }
            }
            None => {
                has_more = false;
            }
        }
    }
    if result.is_empty() {
        Ok((r, None))
    } else {
        Ok((r, Some(result)))
    }
}

pub fn read_one_or_more_while_p<R, F>(predicate: F) -> ReadOneOrMoreWhile<R, F>
where
    R: Reader<Item = char>,
    F: Fn(char) -> bool,
{
    ReadOneOrMoreWhile::new(predicate)
}

pub fn read_zero_or_more_while_p<R, F>(predicate: F) -> impl Parser<R, Output = String>
where
    R: Reader<Item = char>,
    F: Fn(char) -> bool,
{
    read_one_or_more_while_p(predicate).map_none_to_default()
}

// Reads one or more whitespace

pub struct Whitespace<R>(PhantomData<R>);

impl<R> Whitespace<R> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R> Parser<R> for Whitespace<R>
where
    R: Reader<Item = char>,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        do_parse_while(reader, is_whitespace)
    }
}

pub fn read_one_or_more_whitespace_p<R>() -> Whitespace<R>
where
    R: Reader<Item = char>,
{
    Whitespace::new()
}

pub fn read_zero_or_more_whitespace_p<R>() -> MapNoneToDefault<Whitespace<R>>
where
    R: Reader<Item = char>,
{
    read_one_or_more_whitespace_p().map_none_to_default()
}

//
// Concat stuff
//

// Left And Opt Right
// Opt Left And Right

pub struct Stringify<A>(A);

impl<A> Stringify<A> {
    pub fn new(source: A) -> Self {
        Self(source)
    }
}

impl<R, X, Y> Parser<R> for Stringify<LeftAndOptRight<X, Y>>
where
    R: Reader,
    X: Parser<R>,
    Y: Parser<R>,
    X::Output: std::fmt::Display,
    Y::Output: std::fmt::Display,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some((left, Some(right))) => Ok((reader, Some(format!("{}{}", left, right)))),
            Some((left, None)) => Ok((reader, Some(left.to_string()))),
            _ => Ok((reader, None)),
        }
    }
}

impl<R, X, Y> Parser<R> for Stringify<OptLeftAndRight<X, Y>>
where
    R: Reader + Undo<X::Output>,
    X: Parser<R>,
    Y: Parser<R>,
    X::Output: std::fmt::Display,
    Y::Output: std::fmt::Display,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some((Some(left), right)) => Ok((reader, Some(format!("{}{}", left, right)))),
            Some((None, right)) => Ok((reader, Some(right.to_string()))),
            _ => Ok((reader, None)),
        }
    }
}
