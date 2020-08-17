use crate::common::pc::*;
use crate::common::readers::Transactional;

/// An iterator-like trait that iterates over Results.
pub trait ResultIterator {
    type Item;
    type Err;

    /// Gets the next item.
    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>>;
}

// Parser combinators

/// Creates a parser that takes the next element out of the iterator.
pub fn take<I: ResultIterator>() -> impl Fn(&mut I) -> Option<Result<I::Item, I::Err>> {
    |iter| iter.next()
}

/// Takes the first element and maps it with the given mapper function.
/// If the first element cannot be mapped, it returns `None` and the transaction is rolled back.
pub fn take_if_map<I, T, U, E, F>(mapper: F) -> impl Fn(&mut I) -> Option<Result<U, E>>
where
    F: Fn(T) -> Option<U>,
    I: ResultIterator<Item = T, Err = E> + Transactional,
{
    in_transaction_pc(switch(mapper, take()))
}

/// Takes the first element and returns it if it satisfies the given predicate.
/// If the predicate is not met, it returns `None` and the transaction is rolled back.
pub fn take_if_predicate<I: ResultIterator + Transactional, FP>(
    predicate: FP,
) -> impl Fn(&mut I) -> Option<Result<I::Item, I::Err>>
where
    FP: Fn(&I::Item) -> bool,
{
    in_transaction_pc(filter(predicate, take()))
}

/// Takes the first element and returns it unless it satisfies the given predicate.
pub fn take_unless_predicate<I: ResultIterator + Transactional, FP>(
    predicate: FP,
) -> impl Fn(&mut I) -> Option<Result<I::Item, I::Err>>
where
    FP: Fn(&I::Item) -> bool,
{
    in_transaction_pc(exclude(predicate, take()))
}

/// Takes all elements while the predicate is met.
pub fn take_while<I: ResultIterator + Transactional, FP>(
    predicate: FP,
) -> impl Fn(&mut I) -> Option<Result<Vec<I::Item>, I::Err>>
where
    FP: Fn(&I::Item) -> bool,
{
    many(take_if_predicate(predicate))
}

/// Takes all elements until the predicate is met.
pub fn take_until<I: ResultIterator + Transactional, FP>(
    predicate: FP,
) -> impl Fn(&mut I) -> Option<Result<Vec<I::Item>, I::Err>>
where
    FP: Fn(&I::Item) -> bool,
{
    many(take_unless_predicate(predicate))
}

/// Takes the next element and if it is successful (`Some(Ok)`), it
/// maps it to `Some(_)` using the given mapper function.
pub fn take_and_map_to_result<I: ResultIterator, T, F>(
    result_mapper: F,
) -> impl Fn(&mut I) -> Option<Result<T, I::Err>>
where
    F: Fn(I::Item) -> Result<T, I::Err>,
{
    move |lexer| {
        lexer
            .next()
            .and_then(|r| Some(r.and_then(|x| result_mapper(x))))
    }
}
