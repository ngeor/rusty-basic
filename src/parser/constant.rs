use super::Statement;
use crate::common::*;
use crate::parser::assignment;
use crate::parser::char_reader::*;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::Keyword;
use std::io::BufRead;

pub fn constant<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(
        crate::parser::pc::ws::seq2(
            keyword(Keyword::Const),
            demand(
                with_pos(assignment::assignment_tuple()),
                QError::syntax_error_fn("Expected: name"),
            ),
            QError::syntax_error_fn("Expected: whitespace after CONST"),
        ),
        |(
            _,
            Locatable {
                element: (n, e),
                pos,
            },
        )| Statement::Const(n.at(pos), e),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Expression, Statement, TopLevelToken};

    #[test]
    fn parse_const() {
        let input = r#"
        CONST X = 42
        CONST Y$ = "hello"
        "#;
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Const(
                    "X".as_name(2, 15),
                    42.as_lit_expr(2, 19),
                )),
                TopLevelToken::Statement(Statement::Const(
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
                let statement = parse(input).demand_single_statement();
                match statement {
                    Statement::Const(
                        Locatable { element: left, .. },
                        Locatable { element: right, .. },
                    ) => {
                        assert_eq!(left, name.into());
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
                TopLevelToken::Statement(Statement::Const(
                    "ANSWER".as_name(1, 7),
                    42.as_lit_expr(1, 16)
                ))
                .at_rc(1, 1),
                TopLevelToken::Statement(Statement::Comment(
                    " the answer to life, universe, everything".to_string(),
                ))
                .at_rc(1, 19)
            ]
        );
    }
}
