use crate::built_ins;
use crate::common::*;
use crate::lexer::{Keyword, LexemeNode};
use crate::parser::types::*;
use crate::parser::{unexpected, Parser, ParserError, StatementContext};
use std::convert::TryFrom;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_statement(&mut self, next: LexemeNode) -> Result<StatementNode, ParserError> {
        match built_ins::parse_special(self, next, StatementContext::Normal)? {
            built_ins::ParseResult::Ok(x) => Ok(x),
            built_ins::ParseResult::No(next) => match next {
                LexemeNode::Keyword(Keyword::Const, _, pos) => {
                    self.demand_const().map(|x| x.at(pos))
                }
                LexemeNode::Keyword(Keyword::For, _, pos) => {
                    self.demand_for_loop().map(|x| x.at(pos))
                }
                LexemeNode::Keyword(Keyword::GoTo, _, pos) => self
                    .demand_go_to(StatementContext::Normal)
                    .map(|x| x.at(pos)),
                LexemeNode::Keyword(Keyword::If, _, pos) => {
                    self.demand_if_block().map(|x| x.at(pos))
                }
                LexemeNode::Keyword(Keyword::On, _, pos) => self.demand_on().map(|x| x.at(pos)),
                LexemeNode::Keyword(Keyword::Select, _, pos) => {
                    self.demand_select_case().map(|x| x.at(pos))
                }
                LexemeNode::Keyword(Keyword::While, _, pos) => {
                    self.demand_while_block().map(|x| x.at(pos))
                }
                LexemeNode::Symbol('\'', pos) => self.demand_comment().map(|x| x.at(pos)),
                _ => self.demand_assignment_or_sub_call_or_label(next, StatementContext::Normal),
            },
        }
    }

    pub fn demand_comment(&mut self) -> Result<Statement, ParserError> {
        let mut next: LexemeNode = self.buf_lexer.read()?;
        let mut comment: String = String::new();
        while !next.is_eol_or_eof() {
            match next {
                LexemeNode::EOF(_) => {
                    return unexpected("EOF while looking for end of string", next)
                }
                LexemeNode::EOL(_, _) => {
                    return unexpected("Unexpected new line while looking for end of string", next)
                }
                LexemeNode::Keyword(_, s, _)
                | LexemeNode::Word(s, _)
                | LexemeNode::Whitespace(s, _) => comment.push_str(&s),
                LexemeNode::Symbol(c, _) => {
                    comment.push(c);
                }
                LexemeNode::Digits(d, _) => comment.push_str(&format!("{}", d)),
            }
            next = self.buf_lexer.read()?;
        }
        Ok(Statement::Comment(comment))
    }

    pub fn demand_single_line_then_statement(&mut self) -> Result<StatementNode, ParserError> {
        // read bare name
        let next = self.buf_lexer.read()?;
        match built_ins::parse_special(self, next, StatementContext::SingleLineIf)? {
            built_ins::ParseResult::Ok(s) => Ok(s),
            built_ins::ParseResult::No(next) => match next {
                LexemeNode::Word(w, p) => self.demand_assignment_or_sub_call_with_bare_name(
                    CaseInsensitiveString::new(w),
                    p,
                    StatementContext::SingleLineIf,
                ),
                LexemeNode::Keyword(Keyword::GoTo, _, pos) => self
                    .demand_go_to(StatementContext::SingleLineIf)
                    .map(|x| x.at(pos)),
                _ => unexpected("Expected assignment, sub-call or GOTO after THEN", next),
            },
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
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Statement, TopLevelToken};

    #[test]
    fn test_top_level_comment() {
        let input = "' closes the file";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Comment(" closes the file".to_string(),))
                    .at_rc(1, 1)
            ]
        );
    }
}
