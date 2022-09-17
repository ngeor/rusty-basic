use crate::common::QError;
use crate::parser::base::parsers::Parser;
use std::marker::PhantomData;

pub struct StatementSeparator {
    comment_mode: bool,
}

impl StatementSeparator {
    pub fn new(comment_mode: bool) -> Self {
        Self { comment_mode }
    }

    fn parse_comment(&self, reader: R, mut buf: String) -> ReaderResult<R, String, R::Err> {
        let (reader, opt_item) = eol_separator_p().parse(reader)?;
        let item = opt_item.unwrap();
        buf.push_str(item.as_str());
        Ok((reader, Some(buf)))
    }

    // <ws>* '\'' (undoing it)
    // <ws>* ':' <ws*>
    // <ws>* EOL <ws | eol>*
    fn parse_non_comment(&self, reader: R, mut buf: String) -> ReaderResult<R, String, R::Err> {
        let (reader, opt_item) = comment_separator_p()
            .or(colon_separator_p())
            .or(eol_separator_p())
            .parse(reader)?;
        match opt_item {
            Some(item) => {
                buf.push_str(item.as_str());
                Ok((reader, Some(buf)))
            }
            _ => Err((reader, QError::syntax_error("Expected: end-of-statement"))),
        }
    }
}

impl Parser for StatementSeparator {
    type Output = String;

    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        // skip any whitespace, so that the error will hit the first offending character
        let (reader, opt_buf) = whitespace_p().parse(reader)?;
        let buf = opt_buf.unwrap_or_default();
        if self.comment_mode {
            self.parse_comment(reader, buf)
        } else {
            self.parse_non_comment(reader, buf)
        }
    }
}

// '\'' (undoing it)
fn comment_separator_p<R>() -> impl Parser<Output = String> {
    // not adding the ' character in the resulting string because it was already undone
    item_p('\'').peek_reader_item().map(|_| String::new())
}

/// A parser that succeeds on EOF, EOL, colon and comment.
/// Does not undo anything.
pub struct EofOrStatementSeparator;

impl EofOrStatementSeparator {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for EofOrStatementSeparator {
    type Output = String;

    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = reader.read()?;
        match opt_item {
            Some(ch) => {
                if ch == ':' || ch == '\'' || is_eol(ch) {
                    let mut buf: String = String::new();
                    buf.push(ch);
                    Ok((reader, Some(buf)))
                } else {
                    Ok((reader.undo_item(ch), None))
                }
            }
            _ => {
                // EOF is accepted
                Ok((reader, Some(String::new())))
            }
        }
    }
}
