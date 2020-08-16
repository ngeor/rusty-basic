use std::collections::VecDeque;

/// An iterator-like trait that iterates over Results.
pub trait ResultIterator {
    type Item;
    type Err;

    /// Gets the next item.
    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>>;

    /// Creates a ref adaptor around this iterator,
    /// so that it can be used without consuming the original.
    fn to_ref(&mut self) -> RefAdaptor<Self>
    where
        Self: Sized,
    {
        RefAdaptor::new(self)
    }

    /// Collects all items into a `VecDeque`.
    ///
    /// Returns `Ok(VecDeque)` if the iterator does not contain any errors.
    /// Returns `Err(_)` on the first encountered error.
    #[deprecated]
    fn collect(&mut self) -> Result<VecDeque<Self::Item>, Self::Err> {
        let mut result: VecDeque<Self::Item> = VecDeque::new();
        loop {
            match self.next() {
                None => return Ok(result),
                Some(Err(err)) => return Err(err),
                Some(Ok(x)) => result.push_back(x),
            }
        }
    }

    /// Skips the first element in the iterator, calling
    /// the given function with it.
    fn tap_next<F>(&mut self, f: F) -> Skip<Self, F>
    where
        Self: Sized,
        Self::Err: Clone,
        F: FnMut(Self::Item) -> (),
    {
        Skip::new(self, f)
    }

    /// Folds the iterator into a single value using the given accumulator function.
    fn fold<A, F>(&mut self, seed: A, f: F) -> Option<Result<A, Self::Err>>
    where
        Self: Sized,
        F: Fn(A, Self::Item) -> A,
    {
        let mut result: A = seed;
        for x in self.iter() {
            match x {
                Ok(x) => {
                    result = f(result, x);
                }
                Err(err) => {
                    return Some(Err(err));
                }
            }
        }
        Some(Ok(result))
    }

    /// Creates a standard `Iterator` over this `ResultIterator`.
    fn iter(&mut self) -> IteratorAdaptor<Self>
    where
        Self: Sized,
    {
        IteratorAdaptor::new(self)
    }

    /// Gets the first element out of the iterator.
    fn first(mut self) -> Option<Result<Self::Item, Self::Err>>
    where
        Self: Sized,
    {
        self.iter().take(1).next()
    }
}

/// Peek support for an iterator-like trait that iterates over Results.
pub trait PeekResultIterator: ResultIterator {
    /// Peeks the next item.
    ///
    /// The err parameter is not a reference because they always propagate with priority.
    fn peek(&mut self) -> Option<Result<&Self::Item, Self::Err>>;

    /// Checks if the first peeked item satisfies the given predicate
    /// and only then allows the items to be consumed.
    ///
    /// Only the first item is checked with the predicate.
    fn take_if<F>(&mut self, predicate: F) -> TakeIf<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        TakeIf::new(self, predicate)
    }

    /// Reads while the next peeked item satisfies the given predicate.
    fn take_while<F>(&mut self, predicate: F) -> TakeWhile<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        TakeWhile::new(self, predicate)
    }

    /// Reads while the next peeked item can be mapped using the given mapper function.
    fn map_while<F, U>(self, mapper: F) -> MapWhile<Self, F, U>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> Option<Result<U, Self::Err>>,
        U: Sized,
    {
        MapWhile::new(self, mapper)
    }
}

pub struct TakeIf<'a, I, F>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> bool,
{
    iter: &'a mut I,
    predicate: F,
    seen_first: bool,
    passed_condition: bool,
}

impl<'a, I, F> TakeIf<'a, I, F>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> bool,
{
    pub fn new(iter: &'a mut I, predicate: F) -> Self {
        TakeIf {
            iter,
            predicate,
            seen_first: false,
            passed_condition: false,
        }
    }
}

impl<'a, I, F> ResultIterator for TakeIf<'a, I, F>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;
    type Err = I::Err;

    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>> {
        if self.passed_condition {
            // already passed condition, continue passing through
            self.iter.next()
        } else if self.seen_first {
            // not passed condition but seen the first item, stop iteration
            None
        } else {
            // first time
            self.seen_first = true;
            match self.iter.peek() {
                None => None,
                Some(Err(err)) => Some(Err(err)),
                Some(Ok(x)) => {
                    let p = &mut self.predicate;
                    if p(x) {
                        self.passed_condition = true;
                        self.iter.next()
                    } else {
                        None
                    }
                }
            }
        }
    }
}

impl<'a, I, F> PeekResultIterator for TakeIf<'a, I, F>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> bool,
{
    fn peek(&mut self) -> Option<Result<&Self::Item, Self::Err>> {
        if self.passed_condition {
            // already passed condition, continue passing through
            self.iter.peek()
        } else if self.seen_first {
            // not passed condition but seen the first item, stop iteration
            None
        } else {
            // first time
            self.seen_first = true;
            match self.iter.peek() {
                None => None,
                Some(Err(err)) => Some(Err(err)),
                Some(Ok(x)) => {
                    let p = &mut self.predicate;
                    if p(x) {
                        self.passed_condition = true;
                        Some(Ok(x))
                    } else {
                        None
                    }
                }
            }
        }
    }
}

pub struct TakeWhile<'a, I, F>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> bool,
{
    iter: &'a mut I,
    predicate: F,
}

impl<'a, I, F> TakeWhile<'a, I, F>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> bool,
{
    pub fn new(iter: &'a mut I, predicate: F) -> Self {
        TakeWhile { iter, predicate }
    }
}

impl<'a, I, F> ResultIterator for TakeWhile<'a, I, F>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;
    type Err = I::Err;

    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>> {
        match self.iter.peek() {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(x)) => {
                let p = &mut self.predicate;
                if p(x) {
                    self.iter.next()
                } else {
                    None
                }
            }
        }
    }
}

impl<'a, I, F> PeekResultIterator for TakeWhile<'a, I, F>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> bool,
{
    fn peek(&mut self) -> Option<Result<&Self::Item, Self::Err>> {
        match self.iter.peek() {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(x)) => {
                let p = &mut self.predicate;
                if p(x) {
                    Some(Ok(x))
                } else {
                    None
                }
            }
        }
    }
}

pub struct MapWhile<I, F, U>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> Option<Result<U, I::Err>>,
    U: Sized,
{
    iter: I,
    mapper: F,
    last_peeked: Option<U>,
}

impl<I, F, U> MapWhile<I, F, U>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> Option<Result<U, I::Err>>,
    U: Sized,
{
    pub fn new(iter: I, mapper: F) -> Self {
        MapWhile {
            iter,
            mapper,
            last_peeked: None,
        }
    }
}

impl<I, F, U> ResultIterator for MapWhile<I, F, U>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> Option<Result<U, I::Err>>,
    U: Sized,
{
    type Item = U;
    type Err = I::Err;

    fn next(&mut self) -> Option<Result<U, Self::Err>> {
        match self.iter.peek() {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(x)) => {
                let p = &mut self.mapper;
                match p(x) {
                    None => None,
                    Some(Err(err)) => Some(Err(err)),
                    Some(Ok(y)) => {
                        let result = Some(Ok(y));
                        self.iter.next();
                        result
                    }
                }
            }
        }
    }
}

impl<I, F, U> PeekResultIterator for MapWhile<I, F, U>
where
    I: PeekResultIterator,
    F: FnMut(&I::Item) -> Option<Result<U, I::Err>>,
    U: Sized,
{
    fn peek(&mut self) -> Option<Result<&Self::Item, Self::Err>> {
        match self.iter.peek() {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(x)) => {
                let p = &mut self.mapper;
                match p(x) {
                    None => None,
                    Some(Err(err)) => Some(Err(err)),
                    Some(Ok(y)) => {
                        self.last_peeked = Some(y);
                        Some(Ok(self.last_peeked.as_ref().unwrap()))
                    }
                }
            }
        }
    }
}

/// A decorator over an iterator, remembering if it has encountered Err or None.
/// If it encounters Err or None, it will keep returning that result on subsequent calls to `next`.
pub struct Fuse<'a, I>
where
    I: ResultIterator,
    I::Err: Clone,
{
    iter: &'a mut I,
    found_none: bool,
    found_err: Option<I::Err>,
}

impl<'a, I> Fuse<'a, I>
where
    I: ResultIterator,
    I::Err: Clone,
{
    pub fn new(iter: &'a mut I) -> Self {
        Fuse {
            iter,
            found_none: false,
            found_err: None,
        }
    }
}

impl<'a, I> ResultIterator for Fuse<'a, I>
where
    I: ResultIterator,
    I::Err: Clone,
{
    type Item = I::Item;
    type Err = I::Err;

    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>> {
        if self.found_none {
            None
        } else if self.found_err.is_some() {
            Some(Err(self.found_err.clone().unwrap()))
        } else {
            match self.iter.next() {
                None => {
                    self.found_none = true;
                    None
                }
                Some(Err(err)) => {
                    self.found_err = Some(err.clone());
                    Some(Err(err))
                }
                Some(Ok(x)) => Some(Ok(x)),
            }
        }
    }
}

impl<'a, I> PeekResultIterator for Fuse<'a, I>
where
    I: PeekResultIterator,
    I::Err: Clone,
{
    fn peek(&mut self) -> Option<Result<&Self::Item, Self::Err>> {
        if self.found_none {
            None
        } else if self.found_err.is_some() {
            Some(Err(self.found_err.clone().unwrap()))
        } else {
            match self.iter.peek() {
                None => {
                    self.found_none = true;
                    None
                }
                Some(Err(err)) => {
                    self.found_err = Some(err.clone());
                    Some(Err(err))
                }
                Some(Ok(x)) => Some(Ok(x)),
            }
        }
    }
}

/// Skips over the first element, calling a predicate with it
pub struct Skip<'a, I, F>
where
    I: ResultIterator,
    I::Err: Clone,
    F: FnMut(I::Item),
{
    iter: Fuse<'a, I>,
    predicate: F,
    seen_next: bool,
}

impl<'a, I, F> Skip<'a, I, F>
where
    I: ResultIterator,
    I::Err: Clone,
    F: FnMut(I::Item),
{
    pub fn new(iter: &'a mut I, predicate: F) -> Self {
        Skip {
            iter: Fuse::new(iter),
            predicate,
            seen_next: false,
        }
    }

    fn call_predicate(&mut self) {
        self.seen_next = true;
        // we use a Fuse so if we hit None or Error it will remember it
        match self.iter.next() {
            Some(Ok(x)) => {
                let p = &mut self.predicate;
                p(x);
            }
            _ => (),
        }
    }
}

impl<'a, I, F> ResultIterator for Skip<'a, I, F>
where
    I: ResultIterator,
    I::Err: Clone,
    F: FnMut(I::Item),
{
    type Item = I::Item;
    type Err = I::Err;

    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>> {
        if self.seen_next {
            self.iter.next()
        } else {
            self.call_predicate();
            self.next()
        }
    }
}

impl<'a, I, F> PeekResultIterator for Skip<'a, I, F>
where
    I: PeekResultIterator,
    I::Err: Clone,
    F: FnMut(I::Item),
{
    fn peek(&mut self) -> Option<Result<&Self::Item, Self::Err>> {
        if self.seen_next {
            self.iter.peek()
        } else {
            self.call_predicate();
            self.peek()
        }
    }
}

/// An adaptor that converts a `ResultIterator` into a standard `Iterator`.
pub struct IteratorAdaptor<'a, I: ResultIterator> {
    iter: &'a mut I,
}

impl<'a, I: ResultIterator> IteratorAdaptor<'a, I> {
    pub fn new(iter: &'a mut I) -> Self {
        Self { iter }
    }
}

impl<'a, I: ResultIterator> Iterator for IteratorAdaptor<'a, I> {
    type Item = Result<I::Item, I::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// A pass through adaptor that allows to share a reference of an iterator.
pub struct RefAdaptor<'a, I> {
    iter: &'a mut I,
}

impl<'a, I> RefAdaptor<'a, I> {
    pub fn new(iter: &'a mut I) -> Self {
        Self { iter }
    }
}

impl<'a, I: ResultIterator> ResultIterator for RefAdaptor<'a, I> {
    type Item = I::Item;
    type Err = I::Err;

    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>> {
        self.iter.next()
    }
}

impl<'a, I: PeekResultIterator> PeekResultIterator for RefAdaptor<'a, I> {
    fn peek(&mut self) -> Option<Result<&Self::Item, Self::Err>> {
        self.iter.peek()
    }
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
