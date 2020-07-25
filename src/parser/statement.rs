use crate::built_ins;
use crate::common::*;
use crate::lexer::{Keyword, LexemeNode};
use crate::parser::types::*;
use crate::parser::{unexpected, Parser, ParserError};
use std::convert::TryFrom;
use std::io::BufRead;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatementContext {
    Normal,
    SingleLineIf,
}

impl<T: BufRead> Parser<T> {
    pub fn demand_statement(&mut self, next: LexemeNode) -> Result<StatementNode, ParserError> {
        match next {
            LexemeNode::Keyword(Keyword::Const, _, pos) => self.demand_const().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::For, _, pos) => self.demand_for_loop().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::GoTo, _, pos) => self
                .demand_go_to(StatementContext::Normal)
                .map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::If, _, pos) => self.demand_if_block().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::Input, w, pos) => {
                self.demand_input(w, StatementContext::Normal, pos)
            }
            LexemeNode::Keyword(Keyword::Line, _, pos) => {
                self.demand_line_input(StatementContext::Normal, pos)
            }
            LexemeNode::Keyword(Keyword::On, _, pos) => self.demand_on().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::Open, _, pos) => self.demand_open().map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::Name, _, pos) => {
                built_ins::name::demand(self).map(|x| x.at(pos))
            }
            LexemeNode::Keyword(Keyword::Select, _, pos) => {
                self.demand_select_case().map(|x| x.at(pos))
            }
            LexemeNode::Keyword(Keyword::While, _, pos) => {
                self.demand_while_block().map(|x| x.at(pos))
            }
            _ => self.demand_assignment_or_sub_call_or_label(next, StatementContext::Normal),
        }
    }

    pub fn demand_single_line_then_statement(
        &mut self,
        next: LexemeNode,
    ) -> Result<StatementNode, ParserError> {
        // read bare name
        match next {
            LexemeNode::Word(w, p) => self.demand_assignment_or_sub_call_with_bare_name(
                CaseInsensitiveString::new(w),
                p,
                StatementContext::SingleLineIf,
            ),
            LexemeNode::Keyword(Keyword::GoTo, _, pos) => self
                .demand_go_to(StatementContext::SingleLineIf)
                .map(|x| x.at(pos)),
            LexemeNode::Keyword(Keyword::Input, w, pos) => {
                self.demand_input(w, StatementContext::SingleLineIf, pos)
            }
            // TODO accept more built-in subs here e.g. NAME, LINE INPUT
            _ => unexpected("Expected assignment, sub-call or GOTO after THEN", next),
        }
    }

    fn demand_assignment_or_sub_call_or_label(
        &mut self,
        next: LexemeNode,
        context: StatementContext, // don't allow labels if we're doing `IF X THEN Y:`
    ) -> Result<StatementNode, ParserError> {
        // read bare name
        match next {
            LexemeNode::Word(w, p) => self.demand_assignment_or_sub_call_with_bare_name(
                CaseInsensitiveString::new(w),
                p,
                context,
            ),
            _ => unexpected("Expected word for assignment or sub-call", next),
        }
    }

    fn demand_assignment_or_sub_call_with_bare_name(
        &mut self,
        bare_name: CaseInsensitiveString,
        bare_name_pos: Location,
        context: StatementContext, // don't allow labels if we're doing `IF X THEN Y:`
    ) -> Result<StatementNode, ParserError> {
        // next allowed eof, eol, space, equal sign, type qualifier
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::EOF(_) | LexemeNode::EOL(_, _) => {
                self.demand_sub_call(BareNameNode::new(bare_name, bare_name_pos), next, context)
            }
            LexemeNode::Whitespace(_, _) => {
                // not allowed to parse qualifier after space
                self.demand_assignment_or_sub_call_with_bare_name_whitespace(
                    bare_name,
                    bare_name_pos,
                    context,
                )
            }
            LexemeNode::Symbol('=', _) => {
                // assignment, left-side unqualified name node
                self.read_demand_assignment_skipping_whitespace(
                    Name::new_bare(bare_name).at(bare_name_pos),
                    context,
                )
            }
            LexemeNode::Symbol(':', _) => {
                // label
                if context == StatementContext::Normal {
                    self.read_demand_eol_or_eof_skipping_whitespace()?;
                    Ok(Statement::Label(bare_name).at(bare_name_pos))
                } else {
                    unexpected("Expected type qualifier", next)
                }
            }
            LexemeNode::Symbol('(', _) => {
                // parenthesis e.g. Log("message")
                self.demand_sub_call(BareNameNode::new(bare_name, bare_name_pos), next, context)
            }
            LexemeNode::Symbol(ch, _) => match TypeQualifier::try_from(ch) {
                Ok(q) => self.demand_assignment_or_sub_call_with_qualified_name(
                    Name::new_qualified(bare_name, q).at(bare_name_pos),
                    context,
                ),
                Err(_) => unexpected("Expected type qualifier", next),
            },
            _ => unexpected("Syntax error", next),
        }
    }

    fn demand_assignment_or_sub_call_with_bare_name_whitespace(
        &mut self,
        bare_name: CaseInsensitiveString,
        bare_name_pos: Location,
        context: StatementContext,
    ) -> Result<StatementNode, ParserError> {
        // next allowed eof, eol, equal sign
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Symbol('=', _) => self.read_demand_assignment_skipping_whitespace(
                Name::new_bare(bare_name).at(bare_name_pos),
                context,
            ),
            _ => self.demand_sub_call(BareNameNode::new(bare_name, bare_name_pos), next, context),
        }
    }

    fn demand_assignment_or_sub_call_with_qualified_name(
        &mut self,
        name_node: NameNode,
        context: StatementContext,
    ) -> Result<StatementNode, ParserError> {
        // next allowed space and assignment
        let next = self.read_skipping_whitespace()?;
        match next {
            LexemeNode::Symbol('=', _) => {
                self.read_demand_assignment_skipping_whitespace(name_node, context)
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

    fn demand_go_to(&mut self, context: StatementContext) -> Result<Statement, ParserError> {
        self.read_demand_whitespace("Expected space after GOTO")?;
        let name_node = self.read_demand_bare_name_node("Expected label name")?;
        let (name, _) = name_node.consume();
        if context == StatementContext::Normal {
            self.read_demand_eol_or_eof_skipping_whitespace()?;
        }
        Ok(Statement::GoTo(name))
    }

    fn demand_input(
        &mut self,
        raw_name: String,
        context: StatementContext,
        bare_name_pos: Location,
    ) -> Result<StatementNode, ParserError> {
        self.read_demand_whitespace("Expected space after INPUT")?;
        let next = self.buf_lexer.read()?;
        self.demand_sub_call(
            BareNameNode::new(CaseInsensitiveString::new(raw_name), bare_name_pos),
            next,
            context,
        )
    }

    fn demand_line_input(
        &mut self,
        context: StatementContext,
        pos: Location,
    ) -> Result<StatementNode, ParserError> {
        self.read_demand_whitespace("Expected space after LINE")?;
        self.read_demand_keyword(Keyword::Input)?;
        self.read_demand_whitespace("Expected space after INPUT")?;
        let next = self.buf_lexer.read()?;
        self.demand_sub_call(BareNameNode::new("LINE INPUT".into(), pos), next, context)
    }

    fn demand_open(&mut self) -> Result<Statement, ParserError> {
        self.read_demand_whitespace("Expected space after OPEN")?;
        let file_name_expr = self.read_demand_expression()?;
        self.read_demand_whitespace("Expected space after filename")?;
        self.read_demand_keyword(Keyword::For)?;
        self.read_demand_whitespace("Expected space after FOR")?;
        let mode: i32 = self.read_demand_file_mode()?.into();
        self.read_demand_whitespace("Expected space after file mode")?;
        let mut next = self.buf_lexer.read()?;
        let mut access: i32 = FileAccess::Unspecified.into();
        if next.is_keyword(Keyword::Access) {
            self.read_demand_whitespace("Expected space after ACCESS")?;
            access = self.read_demand_file_access()?.into();
            self.read_demand_whitespace("Expected space after file access")?;
            next = self.buf_lexer.read()?;
        }
        if next.is_keyword(Keyword::As) {
            self.read_demand_whitespace("Expected space after AS")?;
            let file_handle = self.read_demand_expression()?;
            let bare_name: BareName = "OPEN".into();

            Ok(Statement::SubCall(
                bare_name,
                vec![
                    file_name_expr,
                    Expression::IntegerLiteral(mode).at(Location::start()),
                    Expression::IntegerLiteral(access).at(Location::start()),
                    file_handle,
                ],
            ))
        } else {
            unexpected("Expected AS", next)
        }
    }

    fn read_demand_file_mode(&mut self) -> Result<FileMode, ParserError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Keyword(Keyword::Input, _, _) => Ok(FileMode::Input),
            LexemeNode::Keyword(Keyword::Output, _, _) => Ok(FileMode::Output),
            LexemeNode::Keyword(Keyword::Append, _, _) => Ok(FileMode::Append),
            _ => unexpected("Expected INPUT|OUTPUT|APPEND after FOR", next),
        }
    }

    fn read_demand_file_access(&mut self) -> Result<FileAccess, ParserError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Keyword(Keyword::Read, _, _) => Ok(FileAccess::Read),
            _ => unexpected("Expected READ after ACCESS", next),
        }
    }
}
