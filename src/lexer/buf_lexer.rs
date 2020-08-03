use super::error::*;
use super::{LexemeNode, Lexer};
use crate::common::*;
use std::convert::From;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

/// Buffering lexer, offering peek/read/undo capabilities.
#[derive(Debug)]
pub struct BufLexer<T: BufRead> {
    lexer: Lexer<T>,
    history: Vec<LexemeNode>,
    index: usize,
    transactions: Vec<usize>,
    last_read: Option<LexemeNode>,
}

impl<T: BufRead> BufLexer<T> {
    pub fn new(lexer: Lexer<T>) -> Self {
        Self {
            lexer,
            history: vec![],
            index: 0,
            transactions: vec![],
            last_read: None,
        }
    }

    pub fn peek(&mut self) -> Result<LexemeNode, LexerError> {
        self.fill_buffer_if_empty()?;
        Ok(self.history[self.index].clone())
    }

    pub fn read(&mut self) -> Result<LexemeNode, LexerError> {
        let result = self.peek()?;
        self.index += 1;
        self.clear_history();
        self.last_read = Some(result.clone()); // TODO 1 test 2 transactions
        Ok(result)
    }

    pub fn begin_transaction(&mut self) {
        self.transactions.push(self.index);
    }

    pub fn undo(&mut self) -> Result<(), LexerError> {
        self.seek(-1)
    }

    pub fn seek(&mut self, offset: i32) -> Result<(), LexerError> {
        if self.transactions.is_empty() {
            Err(LexerError::Internal(
                "Not in transaction".to_string(),
                self.location(),
            ))
        } else {
            let start_index: i32 = self.transactions[self.transactions.len() - 1] as i32;
            let new_index: i32 = (self.index as i32) + offset;
            if new_index >= start_index && new_index < self.history.len() as i32 {
                self.index = new_index as usize;
                Ok(())
            } else {
                Err(LexerError::Internal(
                    "Offset out of range".to_string(),
                    self.location(),
                ))
            }
        }
    }

    pub fn commit_transaction(&mut self) -> Result<(), LexerError> {
        if self.transactions.is_empty() {
            Err(LexerError::Internal(
                "Not in transaction".to_string(),
                self.location(),
            ))
        } else {
            self.transactions.pop();
            Ok(())
        }
    }

    pub fn rollback_transaction(&mut self) -> Result<(), LexerError> {
        if self.transactions.is_empty() {
            Err(LexerError::Internal(
                "Not in transaction".to_string(),
                self.location(),
            ))
        } else {
            self.index = self.transactions.pop().unwrap();
            Ok(())
        }
    }

    fn fill_buffer_if_empty(&mut self) -> Result<(), LexerError> {
        if self.index >= self.history.len() {
            let lexeme = self.lexer.read()?;
            self.history.push(lexeme);
        }
        Ok(())
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

impl<T> From<T> for BufLexer<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        Self::new(input.into())
    }
}

impl From<File> for BufLexer<BufReader<File>> {
    fn from(input: File) -> Self {
        Self::new(input.into())
    }
}

impl<T: BufRead> HasLocation for BufLexer<T> {
    /// Gets the location of the lexeme that will be read next.
    fn location(&self) -> Location {
        if self.index < self.history.len() {
            self.history[self.index].location()
        } else {
            self.lexer.location()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::whitespace(1, 6));
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::digits("1", 1, 7));
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::eof(1, 8));
        assert_eq!(buf_lexer.read().is_err(), true);
    }

    #[test]
    fn test_peek() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        // peek one time
        assert_eq!(buf_lexer.peek().unwrap(), LexemeNode::word("PRINT", 1, 1));
        // peek again should be the same
        assert_eq!(buf_lexer.peek().unwrap(), LexemeNode::word("PRINT", 1, 1));
        // read should be the same
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        // peek should get the next
        assert_eq!(buf_lexer.peek().unwrap(), LexemeNode::whitespace(1, 6));
        // read the next
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::whitespace(1, 6));
        // read the next
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::digits("1", 1, 7));
        // peek should be at eof
        assert_eq!(buf_lexer.peek().unwrap(), LexemeNode::eof(1, 8));
        assert_eq!(buf_lexer.peek().unwrap(), LexemeNode::eof(1, 8));
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::eof(1, 8));
        // peek should also fail after eof has been consumed
        assert_eq!(buf_lexer.peek().is_err(), true);
    }

    #[test]
    fn test_undo_not_in_transaction() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        assert_eq!(
            buf_lexer.undo(),
            Err(LexerError::Internal(
                "Not in transaction".to_string(),
                Location::new(1, 1)
            ))
        );
    }

    #[test]
    fn test_undo_offset_out_of_range() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.undo(),
            Err(LexerError::Internal(
                "Offset out of range".to_string(),
                Location::new(1, 1)
            ))
        );
    }

    #[test]
    fn test_undo() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        buf_lexer.undo().expect("Expected undo to succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
    }

    #[test]
    fn test_commit_transaction_not_in_transaction() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        assert_eq!(
            buf_lexer.commit_transaction(),
            Err(LexerError::Internal(
                "Not in transaction".to_string(),
                Location::new(1, 1)
            ))
        );
    }

    #[test]
    fn test_rollback_transaction_not_in_transaction() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        assert_eq!(
            buf_lexer.rollback_transaction(),
            Err(LexerError::Internal(
                "Not in transaction".to_string(),
                Location::new(1, 1)
            ))
        );
    }

    #[test]
    fn test_commit() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        buf_lexer
            .commit_transaction()
            .expect("commit should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::whitespace(1, 6));
    }

    #[test]
    fn test_commit_transaction_not_in_transaction_after_commit() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        buf_lexer
            .commit_transaction()
            .expect("first commit should succeed");
        assert_eq!(
            buf_lexer.commit_transaction(),
            Err(LexerError::Internal(
                "Not in transaction".to_string(),
                Location::new(1, 6)
            ))
        );
    }

    #[test]
    fn test_rollback() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        buf_lexer
            .rollback_transaction()
            .expect("rollback should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
    }

    #[test]
    fn test_nested_transaction_both_commit() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::whitespace(1, 6));
        buf_lexer
            .commit_transaction()
            .expect("inner commit should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::digits("1", 1, 7));
        buf_lexer
            .commit_transaction()
            .expect("outer commit should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::eof(1, 8));
    }

    #[test]
    fn test_nested_transaction_inner_rollback() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::whitespace(1, 6));
        buf_lexer
            .rollback_transaction()
            .expect("inner rollback should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::whitespace(1, 6));
        buf_lexer
            .commit_transaction()
            .expect("outer commit should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::digits("1", 1, 7));
    }

    #[test]
    fn test_nested_transaction_outer_rollback() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::whitespace(1, 6));
        buf_lexer
            .commit_transaction()
            .expect("inner commit should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::digits("1", 1, 7));
        buf_lexer
            .rollback_transaction()
            .expect("outer rollback should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
    }

    #[test]
    fn test_nested_transaction_both_rollback() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
        buf_lexer.begin_transaction();
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::whitespace(1, 6));
        buf_lexer
            .rollback_transaction()
            .expect("inner rollback should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::whitespace(1, 6));
        buf_lexer
            .rollback_transaction()
            .expect("outer rollback should succeed");
        assert_eq!(buf_lexer.read().unwrap(), LexemeNode::word("PRINT", 1, 1));
    }

    #[test]
    fn test_location() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        assert_eq!(buf_lexer.location(), Location::new(1, 1));
        buf_lexer.read().expect("Read should succeed (PRINT)");
        assert_eq!(buf_lexer.location(), Location::new(1, 6));
        buf_lexer.read().expect("Read should succeed (whitespace)");
        assert_eq!(buf_lexer.location(), Location::new(1, 7));
        buf_lexer.read().expect("Read should succeed (1)");
        assert_eq!(buf_lexer.location(), Location::new(1, 8));
        buf_lexer.read().expect("Read should succeed (EOF)");
        assert_eq!(buf_lexer.location(), Location::new(1, 8));
        buf_lexer.read().expect_err("Read should fail");
    }

    #[test]
    fn test_location_with_peek() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        assert_eq!(buf_lexer.location(), Location::new(1, 1));
        buf_lexer.peek().expect("Peek should succeed");
        assert_eq!(buf_lexer.location(), Location::new(1, 1));
    }

    #[test]
    fn test_location_with_transaction() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        buf_lexer.read().expect("Read should succeed");
        assert_eq!(buf_lexer.location(), Location::new(1, 6));
        buf_lexer
            .rollback_transaction()
            .expect("Rollback should succeed");
        assert_eq!(buf_lexer.location(), Location::new(1, 1));
    }
}
