pub mod binary;
pub mod many;
pub mod text;
pub mod unary;
pub mod unary_fn;

use crate::parser::pc::{Reader, ReaderResult, Undo};
use crate::parser::pc2::unary_fn::{FilterReaderItem, UnaryFnParser};
use std::marker::PhantomData;

pub trait Parser<R>
where
    R: Reader,
{
    type Output;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err>;

    /// For backwards compatibility with the older style fn parsers.
    fn convert_to_fn(self) -> Box<dyn Fn(R) -> ReaderResult<R, Self::Output, R::Err>>
    where
        Self: Sized + 'static,
    {
        let x = self;
        Box::new(move |reader| x.parse(reader))
    }

    /// Wraps this parser into a Box dyn. This is a workaround for dealing with
    /// the compiler's limitations regarding deeply nested concrete parser types.
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

/// Wrapper for older function parsers
pub struct LazyFnParser<R, T, F>(PhantomData<R>, PhantomData<T>, F);

impl<R, T, F> LazyFnParser<R, T, F> {
    pub fn new(f: F) -> Self {
        Self(PhantomData, PhantomData, f)
    }
}

impl<R, T, F> Parser<R> for LazyFnParser<R, T, F>
where
    R: Reader,
    F: Fn() -> Box<dyn Fn(R) -> ReaderResult<R, T, R::Err>>,
{
    type Output = T;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        (self.2)()(reader)
    }
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
