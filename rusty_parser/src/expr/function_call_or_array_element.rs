use rusty_pc::*;

use crate::core::name_as_tokens_p;
use crate::expr::expression_pos_p;
use crate::input::StringView;
use crate::pc_specific::{WithPos, csv, in_parenthesis};
use crate::{ParserError, *};

// function_call ::= <function-name> "(" <expr>* ")"
// function-name ::= <identifier-with-dots>
//                |  <identifier-with-dots> <type-qualifier>
//                |  <keyword> "$"
//
// Cannot invoke function with empty parenthesis, even if they don't have arguments.
// However, it is allowed for arrays, so we parse it.
//
// A function can be qualified.

pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    name_as_tokens_p()
        .and(
            in_parenthesis(csv(expression_pos_p()).or_default()),
            |name_as_tokens: NameAsTokens, arguments: Expressions| {
                Expression::FunctionCall(name_as_tokens.into(), arguments)
            },
        )
        .with_pos()
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::test_utils::*;
    use crate::{assert_expression, *};

    #[test]
    fn test_function_call_expression_one_arg() {
        assert_expression!(
            "IsValid(42)",
            Expression::func("IsValid", vec![42.as_lit_expr(1, 15)])
        );
    }

    #[test]
    fn test_function_call_expression_two_args() {
        assert_expression!(
            "CheckProperty(42, \"age\")",
            Expression::func(
                "CheckProperty",
                vec![42.as_lit_expr(1, 21), "age".as_lit_expr(1, 25)]
            )
        );
    }

    #[test]
    fn test_function_call_in_function_call() {
        assert_expression!(
            "CheckProperty(LookupName(\"age\"), Confirm(1))",
            Expression::func(
                "CheckProperty",
                vec![
                    Expression::func("LookupName", vec!["age".as_lit_expr(1, 32)]).at_rc(1, 21),
                    Expression::func("Confirm", vec![1.as_lit_expr(1, 48)]).at_rc(1, 40)
                ]
            )
        );
    }
}
