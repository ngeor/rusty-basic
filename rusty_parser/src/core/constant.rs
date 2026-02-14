use rusty_pc::*;

use crate::core::name::name_p;
use crate::expr::expression_pos_p;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::equal_sign_ws;
use crate::{Keyword, ParserError, Statement};

pub fn constant_p() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    seq4(
        keyword_ws_p(Keyword::Const),
        name_p().with_pos().or_expected("const name"),
        equal_sign_ws(),
        expression_pos_p().or_expected("const value"),
        |_, const_name, _, const_value_expr| Statement::constant(const_name, const_value_expr),
    )
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::test_utils::*;
    use crate::*;

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
                GlobalStatement::Statement(Statement::constant(
                    "X".as_name(2, 15),
                    42.as_lit_expr(2, 19),
                )),
                GlobalStatement::Statement(Statement::constant(
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
                    Statement::Const(c) => {
                        let (left, right) = c.into();
                        assert_eq!(left.element, Name::from(*name));
                        assert_eq!(right.element, Expression::IntegerLiteral(*value));
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
                GlobalStatement::Statement(Statement::constant(
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
