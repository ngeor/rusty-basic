use super::{Parser, Statement};
use crate::common::Result;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_for_loop(&mut self) -> Result<Option<Statement>> {
        if self.buf_lexer.try_consume_word("FOR")? {
            self.buf_lexer.demand_whitespace()?;
            let for_counter_variable = self.demand_name_with_type_qualifier()?;
            self.buf_lexer.skip_whitespace()?;
            self.buf_lexer.demand_symbol('=')?;
            self.buf_lexer.skip_whitespace()?;
            let lower_bound = self.demand_expression()?;
            self.buf_lexer.demand_whitespace()?;
            self.buf_lexer.demand_specific_word("TO")?;
            self.buf_lexer.demand_whitespace()?;
            let upper_bound = self.demand_expression()?;
            self.buf_lexer.skip_whitespace()?;
            self.buf_lexer.demand_eol()?;
            self.buf_lexer.skip_whitespace_and_eol()?;

            let mut statements: Vec<Statement> = vec![];

            // might have a dummy empty for loop
            while !self.buf_lexer.try_consume_word("NEXT")? {
                statements.push(self.demand_statement()?);
                self.buf_lexer.skip_whitespace_and_eol()?;
            }

            // TODO support "NEXT FOR"
            self.buf_lexer.demand_eol_or_eof()?;

            Ok(Some(Statement::ForLoop(
                for_counter_variable,
                lower_bound,
                upper_bound,
                statements,
            )))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::*;
    use crate::parser::{Expression, QName, Statement, TopLevelToken};

    #[test]
    fn test_for_loop() {
        let input = "FOR I = 1 TO 10\r\nPRINT I\r\nNEXT";
        let result = parse(input);
        assert_eq!(
            result,
            vec![TopLevelToken::Statement(Statement::ForLoop(
                QName::Untyped("I".to_string()),
                Expression::from(1),
                Expression::from(10),
                vec![Statement::sub_call(
                    "PRINT",
                    vec![Expression::variable_name_unqualified("I")]
                )]
            ))]
        );
    }

    #[test]
    fn fn_fixture_for_print_10() {
        let result = parse_file("FOR_PRINT_10.BAS");
        assert_eq!(
            result,
            vec![TopLevelToken::Statement(Statement::ForLoop(
                QName::Untyped("I".to_string()),
                Expression::from(1),
                Expression::from(10),
                vec![Statement::sub_call(
                    "PRINT",
                    vec![
                        Expression::from("Hello"),
                        Expression::variable_name_unqualified("I")
                    ]
                )]
            ))]
        );
    }

    #[test]
    fn fn_fixture_for_nested() {
        let result = parse_file("FOR_NESTED.BAS");
        assert_eq!(
            result,
            vec![
                TopLevelToken::sub_call(
                    "PRINT",
                    vec![Expression::from("Before the outer loop")]
                ),
                TopLevelToken::Statement(Statement::ForLoop(
                    QName::Untyped("I".to_string()),
                    Expression::from(1),
                    Expression::from(10),
                    vec![
                        Statement::sub_call(
                            "PRINT",
                            vec![
                                Expression::from("Before the inner loop"),
                                Expression::variable_name_unqualified("I")
                            ]
                        ),
                        Statement::ForLoop(
                            QName::Untyped("J".to_string()),
                            Expression::from(1),
                            Expression::from(10),
                            vec![Statement::sub_call(
                                "PRINT",
                                vec![
                                    Expression::from("Inner loop"),
                                    Expression::variable_name_unqualified("I"),
                                    Expression::variable_name_unqualified("J"),
                                ]
                            ),]
                        ),
                        Statement::sub_call(
                            "PRINT",
                            vec![
                                Expression::from("After the inner loop"),
                                Expression::variable_name_unqualified("I")
                            ]
                        ),
                    ]
                )),
                TopLevelToken::sub_call(
                    "PRINT",
                    vec![Expression::from("After the outer loop")]
                ),
            ]
        );
    }
}
