use super::{QName, Parser, TopLevelToken};
use crate::common::Result;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_declaration(&mut self) -> Result<Option<TopLevelToken>> {
        if self.buf_lexer.try_consume_word("DECLARE")? {
            match self._parse_declaration() {
                Ok(d) => Ok(Some(d)),
                Err(x) => Err(x),
            }
        } else {
            Ok(None)
        }
    }

    fn _parse_declaration(&mut self) -> Result<TopLevelToken> {
        self.buf_lexer.demand_whitespace()?;
        let next_word = self.buf_lexer.demand_any_word()?;
        if next_word == "FUNCTION" {
            self.buf_lexer.demand_whitespace()?;
            let function_name = self.demand_name_with_type_qualifier()?;
            self.buf_lexer.skip_whitespace()?;
            let function_arguments: Vec<QName> = self.parse_declaration_parameters()?;
            self.buf_lexer.demand_eol_or_eof()?;
            Ok(TopLevelToken::FunctionDeclaration(
                function_name,
                function_arguments,
            ))
        } else {
            Err(format!("Unknown declaration: {}", next_word))
        }
    }

    pub fn parse_declaration_parameters(&mut self) -> Result<Vec<QName>> {
        let mut function_arguments: Vec<QName> = vec![];
        if self.buf_lexer.try_consume_symbol('(')? {
            self.buf_lexer.skip_whitespace()?;
            let mut is_first_parameter = true;
            while !self.buf_lexer.try_consume_symbol(')')? {
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
    use crate::parser::test_utils::*;
    use crate::parser::{QName, TopLevelToken, TypeQualifier};

    #[test]
    fn test_fn() {
        let input = "DECLARE FUNCTION Fib! (N!)";
        let program = parse(input);
        assert_eq!(
            program,
            vec![TopLevelToken::FunctionDeclaration(
                QName::Typed("Fib".to_string(), TypeQualifier::BangSingle),
                vec![QName::Typed("N".to_string(), TypeQualifier::BangSingle)]
            )]
        );
    }
}
