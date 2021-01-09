use std::marker::PhantomData;

use crate::parser::pc::unary_fn::{FilterReaderItem, UnaryFnParser};

pub mod binary;
pub mod many;
pub mod text;
pub mod unary;
pub mod unary_fn;

// TODO use pub self to make a better api surface

pub type ReaderResult<R, T, E> = Result<(R, Option<T>), (R, E)>;

pub trait Reader: Sized {
    type Item;
    type Err;
    fn read(self) -> ReaderResult<Self, Self::Item, Self::Err>;
    fn undo_item(self, item: Self::Item) -> Self;
}

pub trait Undo<T> {
    fn undo(self, item: T) -> Self;
}

pub trait Parser<R>
where
    R: Reader,
{
    type Output;

    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err>;
}

//
// sources
//

macro_rules! source_parser {
    ($name:tt, $fn:tt) => {
        pub struct $name<R: Reader>(PhantomData<R>);

        impl<R: Reader> $name<R> {
            pub fn new() -> Self {
                Self(PhantomData)
            }
        }

        pub fn $fn<R: Reader>() -> $name<R> {
            $name::<R>::new()
        }
    };
}

// the most basic parser, reads anything from the reader
source_parser!(Any, any_p);

impl<R> Parser<R> for Any<R>
where
    R: Reader,
{
    type Output = R::Item;

    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        reader.read()
    }
}

/// A parser that reads the next item from the reader if it meets the given predicate.
pub fn if_p<R, F>(predicate: F) -> FilterReaderItem<Any<R>, F>
where
    R: Reader,
    R::Item: Copy,
    F: Fn(R::Item) -> bool,
{
    any_p::<R>().filter_reader_item(predicate)
}

/// A parser that reads the next item if it matches the given parameter.
pub struct Item<R: Reader>(PhantomData<R>, R::Item);

impl<R> Parser<R> for Item<R>
where
    R: Reader,
    R::Item: Eq,
{
    type Output = R::Item;
    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = reader.read()?;
        match opt_item {
            Some(item) => {
                if item == self.1 {
                    Ok((reader, Some(item)))
                } else {
                    Ok((reader.undo_item(item), None))
                }
            }
            _ => Ok((reader, None)),
        }
    }
}

/// A parser that reads the next item if it matches the given parameter.
pub fn item_p<R>(item: R::Item) -> Item<R>
where
    R: Reader,
    R::Item: Eq,
{
    Item::<R>(PhantomData, item)
}

/// A static parser that always throws an error.
pub struct StaticErrParser<R, T, E>(PhantomData<R>, PhantomData<T>, Option<E>);

impl<R, T, E> Parser<R> for StaticErrParser<R, T, E>
where
    R: Reader<Err = E>,
{
    type Output = T;
    fn parse(&mut self, reader: R) -> ReaderResult<R, T, E> {
        match self.2.take() {
            Some(err) => Err((reader, err)),
            _ => panic!("StaticErrParser cannot be used multiple times"),
        }
    }
}

pub fn static_err_p<R, T, E>(err: E) -> StaticErrParser<R, T, E>
where
    R: Reader<Err = E>,
{
    StaticErrParser(PhantomData, PhantomData, Some(err))
}

pub mod undo {
    use crate::common::Locatable;
    use crate::parser::pc::{Reader, Undo};

    impl<R: Reader<Item = char>> Undo<char> for R {
        fn undo(self, item: char) -> Self {
            self.undo_item(item)
        }
    }

    impl<R: Reader<Item = char>> Undo<Locatable<char>> for R {
        fn undo(self, item: Locatable<char>) -> Self {
            self.undo_item(item.element)
        }
    }

    impl<R: Reader<Item = char>> Undo<String> for R {
        fn undo(self, s: String) -> Self {
            let mut result = self;
            for ch in s.chars().rev() {
                result = result.undo_item(ch);
            }
            result
        }
    }

    impl<R: Reader<Item = char>> Undo<(String, Locatable<char>)> for R {
        fn undo(self, item: (String, Locatable<char>)) -> Self {
            let (a, b) = item;
            self.undo(b).undo(a)
        }
    }

    // undo char followed by opt ws
    impl<R: Reader<Item = char>> Undo<(char, Option<String>)> for R {
        fn undo(self, item: (char, Option<String>)) -> Self {
            let (a, b) = item;
            self.undo(b.unwrap_or_default()).undo_item(a)
        }
    }

    // undo char preceded by opt ws
    impl<B, R: Reader<Item = char> + Undo<String> + Undo<B>> Undo<(Option<String>, B)> for R {
        fn undo(self, item: (Option<String>, B)) -> Self {
            let (a, b) = item;
            self.undo(b).undo(a.unwrap_or_default())
        }
    }
}

pub fn is_whitespace(ch: char) -> bool {
    ch == ' ' || ch == '\t'
}

pub fn is_eol(ch: char) -> bool {
    ch == '\r' || ch == '\n'
}

pub fn is_eol_or_whitespace(ch: char) -> bool {
    is_eol(ch) || is_whitespace(ch)
}

pub fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

pub fn is_letter(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z')
}
