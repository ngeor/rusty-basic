use crate::common::location::*;

/// Reads one item from a stream.
///
/// Returns:
/// - `Ok(Some(item))` if an item is read successfully
/// - `Ok(None)` if the end of stream is found
/// - `Err(err)` if an error occurred when reading the item
pub trait ReadOpt {
    type Item;
    type Err;

    /// Reads one item from a stream.
    ///
    /// Returns:
    /// - `Ok(Some(item))` if an item is read successfully
    /// - `Ok(None)` if the end of stream is found
    /// - `Err(err)` if an error occurred when reading the item
    fn read_ng(&mut self) -> Result<Option<Self::Item>, Self::Err>;
}

/// Peeks one item from a stream, returning a copy of the peeked item.
/// This trait should be implemented when the item implements `Copy`.
pub trait PeekOptCopy: ReadOpt {
    /// Peeks one item from a stream.
    fn peek_ng(&mut self) -> Result<Option<Self::Item>, Self::Err>;
}

/// Peeks one item from a stream, returning a reference of the peeked item.
pub trait PeekOptRef: ReadOpt {
    /// Peeks one item from a stream.
    fn peek_ng(&mut self) -> Result<Option<&Self::Item>, Self::Err>;

    /// Peeks the next item and reads it if it tests successfully against
    /// the given predicate function.
    fn read_if<F>(&mut self, predicate: F) -> Result<Option<Self::Item>, Self::Err>
    where
        F: FnOnce(&Self::Item) -> bool,
    {
        let opt: Option<&Self::Item> = self.peek_ng()?;
        match opt {
            Some(candidate) => {
                if predicate(candidate) {
                    self.read_ng()
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Peeks the next item and if it has a value it passes it through
    /// the given function. If the function returns some result,
    /// the item is read and the value of the function is returned.
    fn map_if<F, U>(&mut self, predicate_mapper: F) -> Result<Option<U>, Self::Err>
    where
        F: FnOnce(&Self::Item) -> Result<Option<U>, Self::Err>,
    {
        let opt: Option<&Self::Item> = self.peek_ng()?;
        match opt {
            Some(candidate) => {
                let mapped = predicate_mapper(candidate)?;
                match mapped {
                    Some(x) => {
                        self.read_ng()?;
                        Ok(Some(x))
                    }
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }
}

///
///  Offers transaction capabilities.
///
pub trait Transactional {
    fn begin_transaction(&mut self);
    fn commit_transaction(&mut self);
    fn rollback_transaction(&mut self);

    /// Runs the given function inside a transaction.
    ///
    /// The transaction is committed only if the function returns `Ok(Some())`,
    /// otherwise it is rolled back.
    fn in_transaction<T, E, F>(&mut self, f: F) -> Result<Option<T>, E>
    where
        F: FnOnce(&mut Self) -> Result<Option<T>, E>,
    {
        self.begin_transaction();
        let result = f(self);
        match &result {
            Ok(Some(_)) => self.commit_transaction(),
            _ => self.rollback_transaction(),
        };
        result
    }
}

#[derive(Debug)]
pub struct TransactionalPeek<R: ReadOpt> {
    reader: R,
    history: Vec<R::Item>,
    index: usize,
    transactions: Vec<usize>,
}

impl<R: ReadOpt> TransactionalPeek<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            history: vec![],
            index: 0,
            transactions: vec![],
        }
    }

    fn fill_buffer_if_empty(&mut self) -> Result<bool, R::Err> {
        if self.index >= self.history.len() {
            match self.reader.read_ng()? {
                Some(x) => {
                    self.history.push(x);
                    Ok(true)
                }
                None => Ok(false),
            }
        } else {
            Ok(true)
        }
    }

    fn clear_history(&mut self) {
        if self.transactions.is_empty() {
            // self.index points to the next possible item in the history buffer
            // remove items from the buffer so that self.index points to zero
            while self.index > 0 && !self.history.is_empty() {
                self.index -= 1;
                self.history.remove(0);
            }
        }
    }
}

impl<R: ReadOpt> ReadOpt for TransactionalPeek<R>
where
    R::Item: Clone,
{
    type Item = R::Item;
    type Err = R::Err;

    fn read_ng(&mut self) -> Result<Option<Self::Item>, Self::Err> {
        let result = self.peek_ng()?.map(|x| x.clone());
        self.index += 1;
        self.clear_history();
        Ok(result)
    }
}

impl<R: ReadOpt> PeekOptRef for TransactionalPeek<R>
where
    R::Item: Clone,
{
    fn peek_ng(&mut self) -> Result<Option<&Self::Item>, Self::Err> {
        if self.fill_buffer_if_empty()? {
            Ok(Some(&self.history[self.index]))
        } else {
            Ok(None)
        }
    }
}

impl<R: ReadOpt> Transactional for TransactionalPeek<R> {
    fn begin_transaction(&mut self) {
        self.transactions.push(self.index);
    }

    fn commit_transaction(&mut self) {
        if self.transactions.is_empty() {
            panic!("Not in transaction");
        } else {
            self.transactions.pop();
        }
    }

    fn rollback_transaction(&mut self) {
        if self.transactions.is_empty() {
            panic!("Not in transaction");
        } else {
            self.index = self.transactions.pop().unwrap();
        }
    }
}

impl<R: ReadOpt + HasLocation> HasLocation for TransactionalPeek<R>
where
    R::Item: HasLocation,
{
    /// Gets the location of the lexeme that will be read next.
    fn pos(&self) -> Location {
        if self.index < self.history.len() {
            self.history[self.index].pos()
        } else {
            self.reader.pos()
        }
    }
}

///
/// Trait for Transactional Peek of R::Item Locatable
///

pub trait LocatableReader<T> {
    type Item;
    type Err;
    fn read_if_ref<F>(&mut self, f: F) -> Result<Option<Self::Item>, Self::Err>
    where
        F: FnOnce(&T) -> bool;

    fn map_if_ref<F, U>(&mut self, f: F) -> Result<Option<Locatable<U>>, Self::Err>
    where
        F: FnOnce(&T) -> Option<U>;
}

impl<T, R: ReadOpt> LocatableReader<T> for TransactionalPeek<R>
where
    R::Item: Clone + HasLocation + AsRef<T>,
{
    type Item = R::Item;
    type Err = R::Err;

    fn read_if_ref<F>(&mut self, f: F) -> Result<Option<R::Item>, R::Err>
    where
        F: FnOnce(&T) -> bool,
    {
        let locatable: Option<&R::Item> = self.peek_ng()?;
        match locatable {
            Some(x) => {
                let inside: &T = x.as_ref();
                let passes: bool = f(inside);
                if passes {
                    self.read_ng()
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    fn map_if_ref<F, U>(&mut self, f: F) -> Result<Option<Locatable<U>>, Self::Err>
    where
        F: FnOnce(&T) -> Option<U>,
    {
        let locatable: Option<&R::Item> = self.peek_ng()?;
        match locatable {
            Some(x) => {
                let inside: &T = x.as_ref();
                match f(inside) {
                    Some(mapped) => {
                        let pos = x.pos();
                        let result = Some(mapped.at(pos));
                        self.read_ng()?;
                        Ok(result)
                    }
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }
}
