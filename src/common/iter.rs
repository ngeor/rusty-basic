/// An iterator-like trait that iterates over Results.
pub trait ResultIterator {
    type Item;
    type Err;

    /// Gets the next item.
    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>>;
}

/// Peek support for an iterator-like trait that iterates over Results.
pub trait PeekResultIterator: ResultIterator {
    /// Peeks the next item.
    ///
    /// The err parameter is not a reference because they always propagate with priority.
    fn peek(&mut self) -> Option<Result<&Self::Item, Self::Err>>;
}

// Parser combinators

/// Takes the first element and maps it with the given mapper function.
/// If the first element cannot be mapped, it returns `None`.
///
/// For efficiency, there is a separate function that tests the peeked result
/// (which has a reference to the item)
/// and a separate function that maps the fetched result (which is owned).
pub fn take_if<I: PeekResultIterator, FP, FM, U>(
    predicate: FP,
    mapper: FM,
) -> impl Fn(&mut I) -> Option<Result<U, I::Err>>
where
    FP: Fn(&I::Item) -> bool,
    FM: Fn(I::Item) -> Option<U>,
    U: Sized,
{
    move |iter| match iter.peek() {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(x)) => {
            if predicate(x) {
                match iter.next() {
                    None => None,
                    Some(Err(err)) => Some(Err(err)),
                    Some(Ok(x)) => match mapper(x) {
                        Some(z) => Some(Ok(z)),
                        None => None,
                    },
                }
            } else {
                None
            }
        }
    }
}

/// Takes all elements while the predicate is met.
pub fn take_while<I: PeekResultIterator, FP>(
    predicate: FP,
) -> impl Fn(&mut I) -> Option<Result<Vec<I::Item>, I::Err>>
where
    FP: Fn(&I::Item) -> bool,
{
    move |iter| {
        let mut result: Vec<I::Item> = vec![];
        loop {
            match iter.peek() {
                None => return Some(Ok(result)),
                Some(Err(err)) => return Some(Err(err)),
                Some(Ok(x)) => {
                    if predicate(x) {
                        match iter.next() {
                            None => return None,
                            Some(Err(err)) => return Some(Err(err)),
                            Some(Ok(x)) => result.push(x),
                        }
                    } else {
                        return Some(Ok(result));
                    }
                }
            }
        }
    }
}

/// Takes all elements until the predicate is met.
pub fn take_until<I: PeekResultIterator, FP>(
    predicate: FP,
) -> impl Fn(&mut I) -> Option<Result<Vec<I::Item>, I::Err>>
where
    FP: Fn(&I::Item) -> bool,
{
    move |iter| {
        let mut result: Vec<I::Item> = vec![];
        loop {
            match iter.peek() {
                None => return Some(Ok(result)),
                Some(Err(err)) => return Some(Err(err)),
                Some(Ok(x)) => {
                    if predicate(x) {
                        return Some(Ok(result));
                    } else {
                        match iter.next() {
                            None => return None,
                            Some(Err(err)) => return Some(Err(err)),
                            Some(Ok(x)) => result.push(x),
                        }
                    }
                }
            }
        }
    }
}
