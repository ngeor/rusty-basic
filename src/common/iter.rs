/// Peek one item from an `Iterator`.
///
/// This mimics the `PeekableIterator` struct of the standard library,
/// but as a trait.
pub trait PeekIter: Iterator {
    fn peek_iter_ng(&mut self) -> Option<&Self::Item>;
}
