use super::{unexpected, BareNameNode, Name, NameNode, Parser, ParserError, TypeQualifier};
use crate::common::*;
use crate::lexer::LexemeNode;
use std::convert::TryFrom;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn read_demand_bare_name_node<S: AsRef<str>>(
        &mut self,
        msg: S,
    ) -> Result<BareNameNode, ParserError> {
        let next = self.buf_lexer.read()?;
        self.demand_bare_name_node(next, msg)
    }

    pub fn read_demand_name_node<S: AsRef<str>>(
        &mut self,
        msg: S,
    ) -> Result<NameNode, ParserError> {
        let next = self.buf_lexer.read()?;
        self.demand_name_node(next, msg)
    }

    pub fn demand_bare_name_node<S: AsRef<str>>(
        &mut self,
        next: LexemeNode,
        msg: S,
    ) -> Result<BareNameNode, ParserError> {
        match next {
            LexemeNode::Word(word, pos) => {
                Ok(BareNameNode::new(CaseInsensitiveString::new(word), pos))
            }
            _ => unexpected(msg, next),
        }
    }

    pub fn demand_name_node<S: AsRef<str>>(
        &mut self,
        next: LexemeNode,
        msg: S,
    ) -> Result<NameNode, ParserError> {
        match next {
            LexemeNode::Word(word, pos) => {
                let optional_type_qualifier = self.try_parse_type_qualifier()?;
                Ok(Name::new(word, optional_type_qualifier).at(pos))
            }
            _ => unexpected(msg, next),
        }
    }

    pub fn try_parse_type_qualifier(&mut self) -> Result<Option<TypeQualifier>, ParserError> {
        self.buf_lexer.try_read(|next| match next {
            LexemeNode::Symbol(ch, _) => TypeQualifier::try_from(*ch),
            _ => Err("Expected qualifier".to_string()),
        })
    }
}
