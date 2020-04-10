use super::*;
use crate::lexer::LexemeNode;
use std::convert::TryFrom;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_name_with_type_qualifier(&mut self) -> Result<Option<NameNode>, LexerError> {
        let next = self.buf_lexer.try_consume_any_word()?;
        match next {
            Some((word, pos)) => {
                let optional_type_qualifier = self.try_parse_type_qualifier()?;
                Ok(Some(NameNode::new(word, optional_type_qualifier, pos)))
            }
            None => Ok(None),
        }
    }

    pub fn demand_name_with_type_qualifier(&mut self) -> Result<NameNode, LexerError> {
        let (name, pos) = self.buf_lexer.demand_any_word()?;
        let optional_type_qualifier = self.try_parse_type_qualifier()?;
        Ok(NameNode::new(name, optional_type_qualifier, pos))
    }

    pub fn try_parse_type_qualifier(&mut self) -> Result<Option<TypeQualifier>, LexerError> {
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
