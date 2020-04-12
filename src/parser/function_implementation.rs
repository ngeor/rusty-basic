use super::{FunctionImplementationNode, NameNode, Parser, TopLevelTokenNode};
use crate::lexer::{Keyword, LexerError};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_function_implementation(
        &mut self,
    ) -> Result<Option<TopLevelTokenNode>, LexerError> {
        let opt_pos = self.buf_lexer.try_consume_keyword(Keyword::Function)?;
        if let Some(pos) = opt_pos {
            // function name
            self.buf_lexer.demand_whitespace()?;
            let name = self.demand_name_with_type_qualifier()?;
            // function parameters
            self.buf_lexer.skip_whitespace()?;
            let function_arguments: Vec<NameNode> = self.parse_declaration_parameters()?;
            self.buf_lexer.demand_eol_or_eof()?;
            let block = self.parse_block()?;
            self.buf_lexer.demand_keyword(Keyword::End)?;
            self.buf_lexer.demand_whitespace()?;
            self.buf_lexer.demand_keyword(Keyword::Function)?;
            self.buf_lexer.demand_eol_or_eof()?;

            Ok(Some(TopLevelTokenNode::FunctionImplementation(
                FunctionImplementationNode::new(name, function_arguments, block, pos),
            )))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::StripLocation;
    use crate::parser::{Expression, Name, Statement, TopLevelToken};

    #[test]
    fn test_function_implementation() {
        let input = "
        FUNCTION Add(A, B)
            Add = A + B
        END FUNCTION
        ";
        let result = parse(input).strip_location();
        assert_eq!(
            result,
            vec![TopLevelToken::FunctionImplementation(
                Name::from("Add"),
                vec![Name::from("A"), Name::from("B")],
                vec![Statement::Assignment(
                    Name::from("Add"),
                    Expression::plus(
                        Expression::VariableName(Name::from("A")),
                        Expression::VariableName(Name::from("B"))
                    )
                )]
            )]
        );
    }

    #[test]
    fn test_function_implementation_lower_case() {
        let input = "
        function add(a, b)
            add = a + b
        end function
        ";
        let result = parse(input).strip_location();
        assert_eq!(
            result,
            vec![TopLevelToken::FunctionImplementation(
                Name::from("add"),
                vec![Name::from("a"), Name::from("b")],
                vec![Statement::Assignment(
                    Name::from("add"),
                    Expression::plus(
                        Expression::VariableName(Name::from("a")),
                        Expression::VariableName(Name::from("b"))
                    )
                )]
            )]
        );
    }
}
