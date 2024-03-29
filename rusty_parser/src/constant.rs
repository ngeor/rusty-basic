use crate::expression::expression_pos_p;
use crate::name;
use crate::pc::*;
use crate::pc_specific::*;
use crate::types::{Keyword, Statement};

pub fn constant_p() -> impl Parser<Output = Statement> {
    seq5(
        keyword(Keyword::Const),
        whitespace().no_incomplete(),
        name::name_with_dots()
            .with_pos()
            .or_syntax_error("Expected: const name"),
        equal_sign().no_incomplete(),
        expression_pos_p().or_syntax_error("Expected: const value"),
        |_, _, const_name, _, const_value_expr| Statement::Const(const_name, const_value_expr),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::*;
    use rusty_common::*;

    #[test]
    fn parse_const() {
        let input = r#"
        CONST X = 42
        CONST Y$ = "hello"
        "#;
        let program = parse_str_no_pos(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::Const(
                    "X".as_name(2, 15),
                    42.as_lit_expr(2, 19),
                )),
                GlobalStatement::Statement(Statement::Const(
                    "Y$".as_name(3, 15),
                    "hello".as_lit_expr(3, 20),
                ))
            ]
        );
    }

    #[test]
    fn parse_numeric_const_dots_in_names() {
        let names = ["A", "AB%", "A.B", "A.B.", "A.%"];
        let values = [-1, 0, 1, 42];
        for name in &names {
            for value in &values {
                let input = format!("CONST {} = {}", name, value);
                let statement = parse(&input).demand_single_statement();
                match statement {
                    Statement::Const(
                        Positioned { element: left, .. },
                        Positioned { element: right, .. },
                    ) => {
                        assert_eq!(left, Name::from(*name));
                        assert_eq!(right, Expression::IntegerLiteral(*value));
                    }
                    _ => panic!("Expected constant"),
                }
            }
        }
    }

    #[test]
    fn test_inline_comment() {
        let input = "CONST ANSWER = 42 ' the answer to life, universe, everything";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::Const(
                    "ANSWER".as_name(1, 7),
                    42.as_lit_expr(1, 16),
                ))
                .at_rc(1, 1),
                GlobalStatement::Statement(Statement::Comment(
                    " the answer to life, universe, everything".to_string(),
                ))
                .at_rc(1, 19)
            ]
        );
    }
}
