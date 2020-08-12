use crate::common::location::*;

///
/// Read one item from a stream
///
pub trait ReadOne {
    type Item;
    type Err;

    fn read(&mut self) -> Result<Option<Self::Item>, Self::Err>;
}

///
/// Peek one item from a stream.
///
pub trait PeekOne: ReadOne {
    fn peek(&mut self) -> Result<Option<&Self::Item>, Self::Err>;

    fn consume_if<F>(&mut self, predicate: F) -> Result<Option<Self::Item>, Self::Err>
    where
        F: FnOnce(&Self::Item) -> bool,
    {
        let opt: Option<&Self::Item> = self.peek()?;
        match opt {
            Some(candidate) => {
                if predicate(candidate) {
                    self.read()
                } else {
                    Ok(None)
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
}

#[derive(Debug)]
pub struct TransactionalPeek<R: ReadOne> {
    reader: R,
    history: Vec<R::Item>,
    index: usize,
    transactions: Vec<usize>,
}

impl<R: ReadOne> TransactionalPeek<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            history: vec![],
            index: 0,
            transactions: vec![],
        }
    }
}

impl<R: ReadOne> ReadOne for TransactionalPeek<R>
where
    R::Item: Clone,
{
    type Item = R::Item;
    type Err = R::Err;

    fn read(&mut self) -> Result<Option<Self::Item>, Self::Err> {
        let result = self.peek()?.map(|x| x.clone());
        self.index += 1;
        self.clear_history();
        Ok(result)
    }
}

impl<R: ReadOne> PeekOne for TransactionalPeek<R>
where
    R::Item: Clone,
{
    fn peek(&mut self) -> Result<Option<&Self::Item>, Self::Err> {
        if self.fill_buffer_if_empty()? {
            Ok(Some(&self.history[self.index]))
        } else {
            Ok(None)
        }
    }
}

impl<R: ReadOne> Transactional for TransactionalPeek<R> {
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

impl<R: ReadOne> TransactionalPeek<R> {
    fn fill_buffer_if_empty(&mut self) -> Result<bool, R::Err> {
        if self.index >= self.history.len() {
            match self.reader.read()? {
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

//
// Locatable reader adapter
//

pub struct LocatableReader<R: ReadOne + HasLocation> {
    reader: R,
}

impl<R: ReadOne + HasLocation> HasLocation for LocatableReader<R> {
    fn pos(&self) -> Location {
        self.reader.pos()
    }
}

impl<R: ReadOne + HasLocation> ReadOne for LocatableReader<R> {
    type Item = Locatable<R::Item>;
    type Err = R::Err;

    fn read(&mut self) -> Result<Option<Self::Item>, Self::Err> {
        let pos = self.reader.pos();
        let next = self.reader.read()?;
        Ok(next.map(|x| x.at(pos)))
    }
}
