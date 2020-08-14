use crate::common::*;
use crate::lexer::*;
use std::convert::From;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

/// BufLexer is a TransactionalPeek over a Lexer
pub type BufLexer<T> = TransactionalPeek<Lexer<T>>;

impl<T: BufRead> BufLexer<T> {
    #[deprecated(note = "please switch to `read_ng`")]
    pub fn read(&mut self) -> Result<LexemeNode, QErrorNode> {
        let pos = self.pos();
        match self.read_ng()? {
            Some(x) => Ok(x),
            None => Ok(Lexeme::EOL("".to_string()).at(pos)),
        }
    }
}

/// Iterator implementation for BufLexer.
/// The implementation if the same for any ReadOpt but it's not possible to implement the trait for a trait.
/// EOF lexeme is turned into a None.
impl<T: BufRead> Iterator for BufLexer<T> {
    type Item = Result<LexemeNode, QErrorNode>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_ng().transpose() {
            Some(Ok(x)) => {
                if x.is_eof() {
                    None
                } else {
                    Some(Ok(x))
                }
            }
            None => None,
            Some(Err(err)) => Some(Err(err)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::whitespace(1, 6)
        );
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::digits("1", 1, 7)
        );
        assert_eq!(buf_lexer.read_ng().unwrap().is_some(), false);
        assert_eq!(buf_lexer.pos(), Location::new(1, 8));
    }

    #[test]
    fn test_peek() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        // peek one time
        assert_eq!(
            buf_lexer.peek_ng().unwrap().unwrap(),
            &LexemeNode::word("PRINT", 1, 1)
        );
        // peek again should be the same
        assert_eq!(
            buf_lexer.peek_ng().unwrap().unwrap(),
            &LexemeNode::word("PRINT", 1, 1)
        );
        // read should be the same
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
        // peek should get the next
        assert_eq!(
            buf_lexer.peek_ng().unwrap().unwrap(),
            &LexemeNode::whitespace(1, 6)
        );
        // read the next
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::whitespace(1, 6)
        );
        // read the next
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::digits("1", 1, 7)
        );
        // peek should be at eof
        assert_eq!(buf_lexer.peek_ng().unwrap().is_some(), false);
        assert_eq!(buf_lexer.peek_ng().unwrap().is_some(), false);
        assert_eq!(buf_lexer.read_ng().unwrap().is_some(), false);
        // peek should also return None after eof has been consumed
        assert_eq!(buf_lexer.peek_ng().unwrap().is_some(), false);
    }

    #[test]
    #[should_panic(expected = "Not in transaction")]
    fn test_commit_transaction_not_in_transaction() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.commit_transaction();
    }

    #[test]
    #[should_panic(expected = "Not in transaction")]
    fn test_rollback_transaction_not_in_transaction() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.rollback_transaction();
    }

    #[test]
    fn test_commit() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
        buf_lexer.commit_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::whitespace(1, 6)
        );
    }

    #[test]
    #[should_panic(expected = "Not in transaction")]
    fn test_commit_transaction_not_in_transaction_after_commit() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
        buf_lexer.commit_transaction();
        buf_lexer.commit_transaction();
    }

    #[test]
    fn test_rollback() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
        buf_lexer.rollback_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
    }

    #[test]
    fn test_nested_transaction_both_commit() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::whitespace(1, 6)
        );
        buf_lexer.commit_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::digits("1", 1, 7)
        );
        buf_lexer.commit_transaction();
        assert_eq!(buf_lexer.read_ng().unwrap().is_some(), false);
        assert_eq!(buf_lexer.pos(), Location::new(1, 8));
    }

    #[test]
    fn test_nested_transaction_inner_rollback() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::whitespace(1, 6)
        );
        buf_lexer.rollback_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::whitespace(1, 6)
        );
        buf_lexer.commit_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::digits("1", 1, 7)
        );
    }

    #[test]
    fn test_nested_transaction_outer_rollback() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::whitespace(1, 6)
        );
        buf_lexer.commit_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::digits("1", 1, 7)
        );
        buf_lexer.rollback_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
    }

    #[test]
    fn test_nested_transaction_both_rollback() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
        buf_lexer.begin_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::whitespace(1, 6)
        );
        buf_lexer.rollback_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::whitespace(1, 6)
        );
        buf_lexer.rollback_transaction();
        assert_eq!(
            buf_lexer.read_ng().unwrap().unwrap(),
            LexemeNode::word("PRINT", 1, 1)
        );
    }

    #[test]
    fn test_location() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        assert_eq!(buf_lexer.pos(), Location::new(1, 1));
        buf_lexer
            .read_ng()
            .unwrap()
            .expect("Read should succeed (PRINT)");
        assert_eq!(buf_lexer.pos(), Location::new(1, 6));
        buf_lexer
            .read_ng()
            .unwrap()
            .expect("Read should succeed (whitespace)");
        assert_eq!(buf_lexer.pos(), Location::new(1, 7));
        buf_lexer
            .read_ng()
            .unwrap()
            .expect("Read should succeed (1)");
        assert_eq!(buf_lexer.pos(), Location::new(1, 8));
        buf_lexer.read_ng().expect("Read should succeed (EOF)");
        assert_eq!(buf_lexer.pos(), Location::new(1, 8));
        buf_lexer
            .read_ng()
            .expect("Read should succeed again (EOF)");
        assert_eq!(buf_lexer.pos(), Location::new(1, 8));
    }

    #[test]
    fn test_location_with_peek() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        assert_eq!(buf_lexer.pos(), Location::new(1, 1));
        buf_lexer.peek_ng().unwrap().expect("Peek should succeed");
        assert_eq!(buf_lexer.pos(), Location::new(1, 1));
    }

    #[test]
    fn test_location_with_transaction() {
        let input = "PRINT 1";
        let mut buf_lexer = BufLexer::from(input);
        buf_lexer.begin_transaction();
        buf_lexer.read_ng().unwrap().expect("Read should succeed");
        assert_eq!(buf_lexer.pos(), Location::new(1, 6));
        buf_lexer.rollback_transaction();
        assert_eq!(buf_lexer.pos(), Location::new(1, 1));
    }
}
