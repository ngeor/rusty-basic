use super::{FunctionDeclarationNode, NameNode, Parser, TopLevelTokenNode};
use crate::common::Location;
use crate::lexer::{LexemeNode, LexerError};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_declaration(&mut self) -> Result<Option<TopLevelTokenNode>, LexerError> {
        match self.buf_lexer.try_consume_word("DECLARE")? {
            Some(pos) => self._parse_declaration(pos).map(|x| Some(x)),
            None => Ok(None),
        }
    }

    fn _parse_declaration(
        &mut self,
        declare_pos: Location,
    ) -> Result<TopLevelTokenNode, LexerError> {
        self.buf_lexer.demand_whitespace()?;
        let (next_word, declarable_pos) = self.buf_lexer.demand_any_word()?;
        if next_word.to_uppercase() == "FUNCTION" {
            self.buf_lexer.demand_whitespace()?;
            let function_name = self.demand_name_with_type_qualifier()?;
            self.buf_lexer.skip_whitespace()?;
            let function_arguments: Vec<NameNode> = self.parse_declaration_parameters()?;
            self.buf_lexer.demand_eol_or_eof()?;
            Ok(TopLevelTokenNode::FunctionDeclaration(
                FunctionDeclarationNode::new(function_name, function_arguments, declare_pos),
            ))
        } else {
            Err(LexerError::Unexpected(
                "Unknown declaration".to_string(),
                LexemeNode::Word(next_word, declarable_pos),
            ))
        }
    }

    pub fn parse_declaration_parameters(&mut self) -> Result<Vec<NameNode>, LexerError> {
        let mut function_arguments: Vec<NameNode> = vec![];
        if self.buf_lexer.try_consume_symbol('(')?.is_some() {
            self.buf_lexer.skip_whitespace()?;
            let mut is_first_parameter = true;
            while self.buf_lexer.try_consume_symbol(')')?.is_none() {
                if is_first_parameter {
                    is_first_parameter = false;
                } else {
                    self.buf_lexer.demand_symbol(',')?;
                    self.buf_lexer.skip_whitespace()?;
                }
                function_arguments.push(self.demand_name_with_type_qualifier()?);
                self.buf_lexer.skip_whitespace()?;
            }
        }
        Ok(function_arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::common::StripLocation;
    use crate::parser::{Name, TopLevelToken};

    #[test]
    fn test_fn() {
        let input = "DECLARE FUNCTION Fib! (N!)";
        let program = parse(input);
        assert_eq!(
            program,
            vec![TopLevelTokenNode::FunctionDeclaration(
                FunctionDeclarationNode::new(
                    NameNode::from("Fib!").at(Location::new(1, 18)),
                    vec![NameNode::from("N!").at(Location::new(1, 24))],
                    Location::new(1, 1)
                )
            )]
        );
    }

    #[test]
    fn test_lower_case() {
        let program = parse("declare function echo$(msg$)").strip_location();
        assert_eq!(
            program,
            vec![TopLevelToken::FunctionDeclaration(
                Name::from("echo$"),
                vec![Name::from("msg$")]
            )]
        )
    }
}
