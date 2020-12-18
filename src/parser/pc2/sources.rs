/// This module holds parses that get data directly from the reader.
use super::{Parser, Reader, ReaderResult};
use std::marker::PhantomData;

macro_rules! source_parser {
    ($name:tt) => {
        pub struct $name<R: Reader>(PhantomData<R>);

        impl<R: Reader> $name<R> {
            pub fn new() -> Self {
                Self(PhantomData)
            }
        }
    };
}

// the most basic parser, reads anything from the reader

source_parser!(Any);

impl<R> Parser<R> for Any<R>
where
    R: Reader,
{
    type Output = R::Item;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        reader.read()
    }
}
