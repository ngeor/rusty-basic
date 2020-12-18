pub mod binary;
pub mod many;
pub mod text;
pub mod unary;
pub mod unary_fn;

use crate::parser::pc::{Reader, ReaderResult, Undo};
use crate::parser::pc2::unary_fn::{FilterReaderItem, UnaryFnParser};
use std::marker::PhantomData;

pub trait Parser<R>: Sized
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

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
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
pub fn item_p<R>(item: R::Item) -> impl Parser<R, Output = R::Item>
where
    R: Reader,
    R::Item: Copy + Eq + 'static,
{
    if_p(move |x| x == item)
}
