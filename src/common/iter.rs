use std::collections::VecDeque;

/// An iterator-like trait that iterates over Results.
pub trait ResultIterator {
    type Item;
    type Err;

    /// Gets the next item.
    fn next(&mut self) -> Option<Result<Self::Item, Self::Err>>;

    /// Collects all items into a `VecDeque`.
    ///
    /// Returns `Ok(VecDeque)` if the iterator does not contain any errors.
    /// Returns `Err(_)` on the first encountered error.
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
}

/// Peek support for an iterator-like trait that iterates over Results.
pub trait PeekResultIterator: ResultIterator + Sized {
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
