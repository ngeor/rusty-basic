use crate::common::*;
use crate::lexer::{Keyword, LexemeNode};
use crate::parser::types::*;
use crate::parser::{unexpected, Parser, ParserError};
use std::convert::TryFrom;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_statement(&mut self, next: LexemeNode) -> Result<StatementNode, ParserError> {
        match next {
            LexemeNode::Keyword(Keyword::For, _, pos) => self.demand_for_loop().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::If, _, pos) => self.demand_if_block().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::While, _, pos) => {
                self.demand_while_block().map(|x| x.at(pos))
            }
            LexemeNode::Keyword(Keyword::Const, _, pos) => self.demand_const().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::On, _, pos) => self.demand_on().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::GoTo, _, pos) => self.demand_go_to().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::Input, w, pos) => self.demand_input(w, pos),
            _ => self.demand_assignment_or_sub_call_or_label(next, true),
        }
    }

    pub fn demand_single_line_then_statement(
        &mut self,
        next: LexemeNode,
    ) -> Result<StatementNode, ParserError> {
        // read bare name
        match next {
            LexemeNode::Word(w, p) => self._demand_assignment_or_sub_call_with_bare_name(
                CaseInsensitiveString::new(w),
                p,
                false,
            ),
            LexemeNode::Keyword(Keyword::GoTo, _, pos) => self.demand_go_to().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::Input, w, pos) => self.demand_input(w, pos),
            _ => unexpected("Expected assignment, sub-call or GOTO after THEN", next),
        }
    }

    fn demand_assignment_or_sub_call_or_label(
        &mut self,
        next: LexemeNode,
        labels_allowed: bool, // don't allow labels if we're doing `IF X THEN Y:`
    ) -> Result<StatementNode, ParserError> {
        // read bare name
        match next {
            LexemeNode::Word(w, p) => self._demand_assignment_or_sub_call_with_bare_name(
                CaseInsensitiveString::new(w),
                p,
                labels_allowed,
            ),
            _ => unexpected("Expected word for assignment or sub-call", next),
        }
    }

    fn _demand_assignment_or_sub_call_with_bare_name(
        &mut self,
        bare_name: CaseInsensitiveString,
        bare_name_pos: Location,
        labels_allowed: bool, // don't allow labels if we're doing `IF X THEN Y:`
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
                self.read_demand_assignment_skipping_whitespace(
                    Name::new_bare(bare_name).at(bare_name_pos),
                )
            }
            LexemeNode::Symbol(':', _) => {
                // label
                if labels_allowed {
                    self.read_demand_eol_or_eof_skipping_whitespace()?;
                    Ok(Statement::Label(bare_name).at(bare_name_pos))
                } else {
                    unexpected("Expected type qualifier", next)
                }
            }
            LexemeNode::Symbol(ch, _) => match TypeQualifier::try_from(ch) {
                Ok(q) => self._demand_assignment_or_sub_call_with_qualified_name(
                    Name::new_qualified(bare_name, q).at(bare_name_pos),
                ),
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
                Name::new_bare(bare_name).at(bare_name_pos),
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
    ) -> Result<(StatementNodes, LexemeNode), ParserError>
    where
        F: Fn(&LexemeNode) -> bool,
    {
        let mut statements: StatementNodes = vec![];
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

    fn demand_on(&mut self) -> Result<Statement, ParserError> {
        self.read_demand_whitespace("Expected space after ON")?;
        self.read_demand_keyword(Keyword::Error)?;
        self.read_demand_whitespace("Expected space after ERROR")?;
        self.read_demand_keyword(Keyword::GoTo)?;
        self.read_demand_whitespace("Expected space after GOTO")?;
        let name_node = self.read_demand_bare_name_node("Expected label name")?;
        let (name, _) = name_node.consume();
        Ok(Statement::ErrorHandler(name))
    }

    fn demand_go_to(&mut self) -> Result<Statement, ParserError> {
        self.read_demand_whitespace("Expected space after GOTO")?;
        let name_node = self.read_demand_bare_name_node("Expected label name")?;
        let (name, _) = name_node.consume();
        Ok(Statement::GoTo(name))
    }

    fn demand_input(
        &mut self,
        raw_name: String,
        bare_name_pos: Location,
    ) -> Result<StatementNode, ParserError> {
        self.read_demand_whitespace("Expected space after INPUT")?;
        let next = self.buf_lexer.read()?;
        self.demand_sub_call(
            BareNameNode::new(CaseInsensitiveString::new(raw_name), bare_name_pos),
            next,
        )
    }
}
