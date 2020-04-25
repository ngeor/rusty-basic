use super::{
    unexpected, BareNameNode, BlockNode, Name, NameNode, Parser, ParserError, QualifiedName,
    StatementNode, TypeQualifier,
};
use crate::common::{CaseInsensitiveString, Location};
use crate::lexer::{Keyword, LexemeNode};
use std::convert::TryFrom;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_statement(&mut self, next: LexemeNode) -> Result<StatementNode, ParserError> {
        match next {
            LexemeNode::Keyword(Keyword::For, _, pos) => self.demand_for_loop(pos),
            LexemeNode::Keyword(Keyword::If, _, pos) => self.demand_if_block(pos),
            LexemeNode::Keyword(Keyword::While, _, pos) => self.demand_while_block(pos),
            LexemeNode::Keyword(Keyword::Const, _, pos) => self.demand_const(pos),
            _ => self.demand_assignment_or_sub_call(next),
        }
    }

    pub fn demand_assignment_or_sub_call(
        &mut self,
        next: LexemeNode,
    ) -> Result<StatementNode, ParserError> {
        // read bare name
        match next {
            LexemeNode::Word(w, p) => {
                self._demand_assignment_or_sub_call_with_bare_name(CaseInsensitiveString::new(w), p)
            }
            _ => unexpected("Expected word for assignment or sub-call", next),
        }
    }

    fn _demand_assignment_or_sub_call_with_bare_name(
        &mut self,
        bare_name: CaseInsensitiveString,
        bare_name_pos: Location,
    ) -> Result<StatementNode, ParserError> {
        // next allowed eof, eol, space, equal sign, type qualifier
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::EOF(_) | LexemeNode::EOL(_, _) => {
                self.demand_sub_call(BareNameNode::new(bare_name, bare_name_pos), next)
            }
            LexemeNode::Whitespace(_, _) => {
                // not allowed to parse qualifier after space
                self._demand_assignment_or_sub_call_with_bare_name_whitespace(
                    bare_name,
                    bare_name_pos,
                )
            }
            LexemeNode::Symbol('=', _) => {
                // assignment, left-side unqualified name node
                self.read_demand_assignment_skipping_whitespace(NameNode::new(
                    Name::Bare(bare_name),
                    bare_name_pos,
                ))
            }
            LexemeNode::Symbol(ch, _) => match TypeQualifier::try_from(ch) {
                Ok(q) => self._demand_assignment_or_sub_call_with_qualified_name(NameNode::new(
                    Name::Typed(QualifiedName::new(bare_name, q)),
                    bare_name_pos,
                )),
                Err(_) => unexpected("Expected type qualifier", next),
            },
            _ => unexpected("Syntax error", next),
        }
    }

    fn _demand_assignment_or_sub_call_with_bare_name_whitespace(
        &mut self,
        bare_name: CaseInsensitiveString,
        bare_name_pos: Location,
    ) -> Result<StatementNode, ParserError> {
        // next allowed eof, eol, equal sign
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Symbol('=', _) => self.read_demand_assignment_skipping_whitespace(
                NameNode::new(Name::Bare(bare_name), bare_name_pos),
            ),
            _ => self.demand_sub_call(BareNameNode::new(bare_name, bare_name_pos), next),
        }
    }

    fn _demand_assignment_or_sub_call_with_qualified_name(
        &mut self,
        name_node: NameNode,
    ) -> Result<StatementNode, ParserError> {
        // next allowed space and assignment
        let next = self.read_skipping_whitespace()?;
        match next {
            LexemeNode::Symbol('=', _) => {
                self.read_demand_assignment_skipping_whitespace(name_node)
            }
            _ => unexpected("Syntax error", next),
        }
    }

    pub fn parse_statements<F, S: AsRef<str>>(
        &mut self,
        exit_predicate: F,
        eof_msg: S,
    ) -> Result<(BlockNode, LexemeNode), ParserError>
    where
        F: Fn(&LexemeNode) -> bool,
    {
        let mut statements: BlockNode = vec![];
        loop {
            let next = self.read_skipping_whitespace_and_eol()?;
            if next.is_eof() {
                return unexpected(eof_msg, next);
            }
            let found_exit = exit_predicate(&next);
            if found_exit {
                return Ok((statements, next));
            }

            statements.push(self.demand_statement(next)?);
        }
    }
}
