use super::{unexpected, BlockNode, Parser, ParserError, StatementNode};
use crate::lexer::{Keyword, LexemeNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_statement(&mut self, next: LexemeNode) -> Result<StatementNode, ParserError> {
        match next {
            LexemeNode::Keyword(Keyword::For, _, pos) => self.demand_for_loop(pos),
            LexemeNode::Keyword(Keyword::If, _, pos) => self.demand_if_block(pos),
            LexemeNode::Keyword(Keyword::While, _, pos) => self.demand_while_block(pos),
            _ => self.demand_single_line_statement(next),
        }
    }

    pub fn demand_single_line_statement(
        &mut self,
        next: LexemeNode,
    ) -> Result<StatementNode, ParserError> {
        let name =
            self.demand_name_with_type_qualifier(next, "Expected word for single line statement")?;
        let (opt_space, next) = self.read_preserve_whitespace()?;
        match next {
            LexemeNode::Symbol('=', _) => {
                // assignment
                self.read_demand_assignment_skipping_whitespace(name)
            }
            LexemeNode::EOL(_, _) | LexemeNode::EOF(_) => self.demand_sub_call(name, next),
            _ => {
                match opt_space {
                    Some(_) => self.demand_sub_call(name, next),
                    None => {
                        // wtf
                        unexpected("Syntax error", next)
                    }
                }
            }
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
