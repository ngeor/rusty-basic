/// Peek one item from an `Iterator`.
///
/// This mimics the `PeekableIterator` struct of the standard library,
/// but as a trait.
pub trait PeekIterRef: Iterator {
    fn peek_iter_ng(&mut self) -> Option<&Self::Item>;
}

/// Peek one item from an `Iterator`, returning a copy of the item.
///
/// This should be implemented only when the item implements `Copy`.
///
/// This mimics the `PeekableIterator` struct of the standard library,
/// but as a trait.
pub trait PeekIterCopy: Iterator {
    fn peek_iter_ng(&mut self) -> Option<Self::Item>;
}
