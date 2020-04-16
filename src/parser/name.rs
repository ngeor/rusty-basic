use super::{NameNode, Parser, ParserError, TypeQualifier};
use crate::common::Location;
use crate::lexer::LexemeNode;
use std::convert::TryFrom;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_name_with_type_qualifier(&mut self) -> Result<Option<NameNode>, ParserError> {
        let next = self.buf_lexer.try_consume_any_word()?;
        match next {
            Some((word, pos)) => self._parse(word, pos).map(|x| Some(x)),
            None => Ok(None),
        }
    }

    pub fn demand_name_with_type_qualifier(&mut self) -> Result<NameNode, ParserError> {
        let (name, pos) = self.buf_lexer.demand_any_word()?;
        self._parse(name, pos)
    }

    fn _parse(&mut self, word: String, pos: Location) -> Result<NameNode, ParserError> {
        let optional_type_qualifier = self.try_parse_type_qualifier()?;
        Ok(NameNode::from(word, optional_type_qualifier, pos))
    }

    pub fn try_parse_type_qualifier(&mut self) -> Result<Option<TypeQualifier>, ParserError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Symbol(ch, _) => match TypeQualifier::try_from(ch) {
                Ok(t) => {
                    self.buf_lexer.consume();
                    Ok(Some(t))
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
}
