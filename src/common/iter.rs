use std::collections::VecDeque;

/// An iterator-like trait that iterates over Results.
pub trait ResultIterator: Sized {
    type Item;
    type Err;

    /// Gets the next item.
    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>>;

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
        Self::Err: Clone,
        F: FnMut(Self::Item) -> (),
    {
        Skip::new(self, f)
    }

    /// Folds the iterator into a single value using the given accumulator function.
    fn fold<A, F>(&mut self, seed: A, f: F) -> Option<Result<A, Self::Err>>
    where
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
    fn iter(&mut self) -> IteratorAdaptor<Self> {
        IteratorAdaptor::new(self)
    }
}

/// Peek support for an iterator-like trait that iterates over Results.
pub trait PeekResultIterator: ResultIterator {
    /// Peeks the next item.
    fn peek(&mut self) -> Option<Result<&Self::Item, Self::Err>>; // errors are always copied because they have precedence

    /// Checks if the first peeked item satisfies the given predicate
    /// and only then allows the items to be consumed.
    ///
    /// Only the first item is checked with the predicate.
    fn take_if<F>(&mut self, predicate: F) -> TakeIf<Self, F>
    where
        F: FnMut(&Self::Item) -> bool,
    {
        TakeIf::new(self, predicate)
    }

    /// Reads while the next peeked item satisfies the given predicate.
    fn take_while<F>(&mut self, predicate: F) -> TakeWhile<Self, F>
    where
        F: FnMut(&Self::Item) -> bool,
    {
        TakeWhile::new(self, predicate)
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
