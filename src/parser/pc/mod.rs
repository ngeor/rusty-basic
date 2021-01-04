use std::marker::PhantomData;

use crate::parser::pc::unary_fn::{FilterReaderItem, UnaryFnParser};

pub mod binary;
pub mod many;
pub mod text;
pub mod unary;
pub mod unary_fn;

pub trait Parser<R>
where
    R: Reader,
{
    type Output;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err>;

    /// Wraps this parser into a Box dyn. This is a workaround for dealing with
    /// the compiler's limitations regarding deeply nested concrete parser types.
    //#[deprecated]
    fn box_dyn(self) -> BoxDynParser<R, Self::Output>
    where
        Self: Sized + 'static,
    {
        BoxDynParser {
            source: Box::new(self),
        }
    }
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

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
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
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
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

/// A static parser that returns the given item, without reading from the reader.
pub struct StaticParser<R, T>(PhantomData<R>, T);

impl<R, T> Parser<R> for StaticParser<R, T>
where
    R: Reader,
    T: Clone,
{
    type Output = T;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        Ok((reader, Some(self.1.clone())))
    }
}

pub fn static_p<R, T>(item: T) -> StaticParser<R, T> {
    StaticParser(PhantomData, item)
}

/// A static parser that returns the given optional item, without reading from the reader.
pub struct OptStaticParser<R, T>(PhantomData<R>, Option<T>);

impl<R, T> Parser<R> for OptStaticParser<R, T>
where
    R: Reader,
    T: Clone,
{
    type Output = T;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        Ok((reader, self.1.clone()))
    }
}

pub fn opt_static_p<R, T>(item: Option<T>) -> OptStaticParser<R, T> {
    OptStaticParser(PhantomData, item)
}

/// A static parser which always returns `None`.
pub struct NoneParser<R, T>(PhantomData<R>, PhantomData<T>);

impl<R, T> Parser<R> for NoneParser<R, T>
where
    R: Reader,
{
    type Output = T;
    fn parse(&self, reader: R) -> ReaderResult<R, T, R::Err> {
        Ok((reader, None))
    }
}

pub fn static_none_p<R, T>() -> NoneParser<R, T>
where
    R: Reader,
{
    NoneParser(PhantomData, PhantomData)
}

/// A static parser that always throws an error.
pub struct StaticErrParser<R, T, E>(PhantomData<R>, PhantomData<T>, E);

impl<R, T, E> Parser<R> for StaticErrParser<R, T, E>
where
    R: Reader<Err = E>,
    E: Clone,
{
    type Output = T;
    fn parse(&self, reader: R) -> ReaderResult<R, T, E> {
        // TODO: it should be possible to remove cloning in the static parsers by adding mutability (which is currently blocked due to deprecated parsers)
        Err((reader, self.2.clone()))
    }
}

pub fn static_err_p<R, T, E>(err: E) -> StaticErrParser<R, T, E>
where
    R: Reader<Err = E>,
    E: Clone,
{
    StaticErrParser(PhantomData, PhantomData, err)
}

/// A workaround parser that wraps a parser into a box.
/// This works around the compiler's limitations dealing with too deeply nested
/// concrete parser types.
pub struct BoxDynParser<R, T> {
    source: Box<dyn Parser<R, Output = T>>,
}

impl<R, T> Parser<R> for BoxDynParser<R, T>
where
    R: Reader,
{
    type Output = T;
    fn parse(&self, reader: R) -> ReaderResult<R, T, R::Err> {
        self.source.parse(reader)
    }
}

pub type ReaderResult<R, T, E> = Result<(R, Option<T>), (R, E)>;

pub trait Undo<T> {
    fn undo(self, item: T) -> Self;
}

pub trait Reader: Sized {
    type Item;
    type Err;
    fn read(self) -> ReaderResult<Self, Self::Item, Self::Err>;
    fn undo_item(self, item: Self::Item) -> Self;
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