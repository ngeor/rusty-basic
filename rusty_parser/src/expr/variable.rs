// TODO consider nesting variable/function_call modules inside property as they are only used there
use std::collections::VecDeque;

use rusty_pc::*;

use crate::core::{name_as_tokens_p, token_to_type_qualifier};
use crate::input::StringView;
use crate::pc_specific::WithPos;
use crate::{
    BareName, Expression, ExpressionPos, ExpressionType, Name, NameAsTokens, ParserError, VariableInfo
};

// variable ::= <identifier-with-dots>
//           |  <identifier-with-dots> <type-qualifier>
//           |  <keyword> "$"
//
// must not be followed by parenthesis (solved by ordering of parsers)
//
// if <identifier-with-dots> contains dots, it might be converted to a property expression
pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    name_as_tokens_p().map(map_to_expr).with_pos()
}

fn map_to_expr(name_as_tokens: NameAsTokens) -> Expression {
    if is_property_expr(&name_as_tokens) {
        map_to_property(name_as_tokens)
    } else {
        Expression::Variable(name_as_tokens.into(), VariableInfo::unresolved())
    }
}

fn is_property_expr(name_as_tokens: &NameAsTokens) -> bool {
    let (name_token, _) = name_as_tokens;
    let mut name_count = 1;
    let mut last_was_dot = false;

    // leading dot cannot happen
    debug_assert!(!name_token.as_str().starts_with('.'));

    for name in name_token.as_str().chars() {
        if '.' == name {
            if last_was_dot {
                // two dots in a row
                return false;
            } else {
                last_was_dot = true;
            }
        } else {
            if last_was_dot {
                name_count += 1;
                last_was_dot = false;
            }
        }
    }
    // at least two names and no trailing dot
    name_count > 1 && !last_was_dot
}

fn map_to_property(name_as_tokens: NameAsTokens) -> Expression {
    let (name_token, opt_q_token) = name_as_tokens;
    let mut property_names: VecDeque<String> = name_token
        .as_str()
        .split('.')
        .map(|s| s.to_owned())
        .collect();
    let mut result = Expression::Variable(
        Name::bare(BareName::new(property_names.pop_front().unwrap())),
        VariableInfo::unresolved(),
    );
    while let Some(property_name) = property_names.pop_front() {
        let is_last = property_names.is_empty();
        let opt_q_next = if is_last {
            opt_q_token.as_ref().map(token_to_type_qualifier)
        } else {
            None
        };
        result = Expression::Property(
            Box::new(result),
            Name::new(BareName::new(property_name), opt_q_next),
            ExpressionType::Unresolved,
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::test_utils::*;
    use crate::{assert_expression, assert_parser_err, expr, *};

    #[test]
    fn test_bare_name() {
        assert_expression!("A", Expression::var_unresolved("A"));
    }

    #[test]
    fn test_bare_name_with_elements() {
        assert_expression!(
            "A.B",
            Expression::Property(
                Box::new(Expression::var_unresolved("A")),
                "B".into(),
                ExpressionType::Unresolved
            )
        );
    }

    #[test]
    fn test_qualified_name() {
        assert_expression!("A%", Expression::var_unresolved("A%"));
    }

    #[test]
    fn test_array() {
        assert_expression!("choice$()", Expression::func("choice$", vec![]));
    }

    #[test]
    fn test_array_element_single_dimension() {
        assert_expression!(
            "choice$(1)",
            Expression::func("choice$", vec![1.as_lit_expr(1, 15)])
        );
    }

    #[test]
    fn test_array_element_two_dimensions() {
        assert_expression!(
            "choice$(1, 2)",
            Expression::func("choice$", vec![1.as_lit_expr(1, 15), 2.as_lit_expr(1, 18)])
        );
    }

    #[test]
    fn test_array_element_user_defined_type() {
        assert_expression!(
            "cards(1).Value",
            Expression::Property(
                Box::new(Expression::func("cards", vec![1.as_lit_expr(1, 13)])),
                "Value".into(),
                ExpressionType::Unresolved
            )
        );
    }

    #[test]
    fn test_array_element_function_call_as_dimension() {
        assert_expression!(
            "cards(lbound(cards) + 1).Value",
            Expression::Property(
                Box::new(Expression::func(
                    "cards",
                    vec![
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new(
                                Expression::func("lbound", vec!["cards".as_var_expr(1, 20)])
                                    .at_rc(1, 13)
                            ),
                            Box::new(1.as_lit_expr(1, 29)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 27)
                    ]
                )),
                "Value".into(),
                ExpressionType::Unresolved
            )
        );
    }

    #[cfg(test)]
    mod name {
        use rusty_pc::Parser;

        use super::*;
        use crate::expr::expression_pos_p;
        use crate::input::create_string_tokenizer;

        #[test]
        fn test_var_unresolved() {
            let inputs = ["abc", "abc.", "abc..", "abc$", "abc.$", "abc..$"];
            for input in inputs {
                assert_expression!(var input);
            }
        }

        #[test]
        fn test_func() {
            let inputs = ["A(1)", "A$(1)", "a.b$(1)"];
            for input in inputs {
                assert_expression!(fn input);
            }
        }

        #[test]
        fn test_possible_property() {
            let input = "a.b.c";
            assert_expression!(input, expr!(prop("a", "b", "c")));
        }

        #[test]
        fn test_bare_array_bare_property() {
            let input = "A(1).Suit";
            assert_expression!(
                input,
                expr!(prop(expr!(fn "A", 1.as_lit_expr(1,9)), "Suit"))
            );
        }

        #[test]
        fn test_bare_array_qualified_property() {
            let input = "A(1).Suit$";
            assert_expression!(
                input,
                expr!(prop(expr!(fn "A", 1.as_lit_expr(1,9)), "Suit$"))
            );
        }

        #[test]
        fn test_possible_qualified_property() {
            let input = "a.b$";
            assert_expression!(input, expr!(prop("a"."b$")));
        }

        #[test]
        fn test_left_side_bare_array_cannot_have_consecutive_dots_in_properties() {
            let inputs = ["A(1).O..ops", "A(1).O..ops = 1"];
            for input in inputs {
                assert_parser_err!(input, "Expected: identifier");
            }
        }

        #[test]
        fn test_left_side_expected_equals() {
            let inputs = [
                "abc$",       // this one would expect 'variable=expression' in QBasic
                "abc$%",      // this one would expect 'variable=expression' in QBasic
                "abc%% = 42", // this one would expect 'variable=expression' in QBasic
                // trailing dot
                "A$.",
                "A$.B",
                "A$. = \"hello\"",
                // property of qualified array
                "A$(1).Oops",
                "A$(1).Oops = 42",
                // trailing dot on qualified array property
                "A(1).Suits$.",
                "A(1).Suits$. = \"hi\"",
                // extra qualifier on qualified array property
                "A(1).Suits$%",
                "A(1).Suits$% = 42",
            ];
            for input in inputs {
                assert_parser_err!(input, expected("="));
            }
        }

        #[test]
        fn test_right_side_expected_end_of_statement() {
            let inputs = [
                // double qualifier
                "Help A%%",
                // property of qualified array
                "Help A$(1).Oops",
                // trailing dot on qualified array property
                "Help A(1).Suits$.",
                // extra qualifier on qualified array property
                "Help A(1).Suits$%",
            ];
            for input in inputs {
                assert_parser_err!(input, expected("end-of-statement"));
            }
        }

        mod test_with_expression_parser {
            use rusty_pc::InputTrait;

            use super::*;

            #[test]
            fn test_double_qualifier() {
                let input = "A%%";
                let mut reader = create_string_tokenizer(input.to_owned());
                let mut parser = expression_pos_p();
                let expr = parser.parse(&mut reader).ok().unwrap();
                assert!(!reader.is_eof());
                let expr = expr.element();
                assert!(matches!(expr, Expression::Variable(_, _)));
                if let Expression::Variable(n, _) = expr {
                    assert_eq!(
                        n,
                        Name::qualified("A".into(), TypeQualifier::PercentInteger)
                    );
                }
            }
            #[test]
            fn test_trailing_dot_after_qualifier() {
                let input = "A$.";
                let mut reader = create_string_tokenizer(input.to_owned());
                let mut parser = expression_pos_p();
                let expr = parser.parse(&mut reader).ok().unwrap();
                assert!(!reader.is_eof());
                let expr = expr.element();
                assert!(matches!(expr, Expression::Variable(_, _)));
                if let Expression::Variable(n, _) = expr {
                    assert_eq!(n, Name::qualified("A".into(), TypeQualifier::DollarString));
                }
            }
            #[test]
            fn test_property_of_qualified_array() {
                let input = "A$(1).Oops";
                let mut reader = create_string_tokenizer(input.to_owned());
                let mut parser = expression_pos_p();
                let expr = parser.parse(&mut reader).ok().unwrap();
                assert!(!reader.is_eof());
                let expr = expr.element();
                assert!(matches!(expr, Expression::FunctionCall(_, _)));
                if let Expression::FunctionCall(n, _) = expr {
                    assert_eq!(n, Name::qualified("A".into(), TypeQualifier::DollarString));
                }
            }
        }
    }
}
