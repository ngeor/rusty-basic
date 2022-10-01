pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;
    use crate::parser::expression::{back_guarded_expression_node_p, guarded_expression_node_p};

    pub fn parse() -> impl Parser<Output = Statement> {
        keyword_pair(Keyword::View, Keyword::Print)
            .and_opt(parse_args())
            .keep_right()
            .map(|opt_args| {
                Statement::BuiltInSubCall(BuiltInSub::ViewPrint, opt_args.unwrap_or_default())
            })
    }

    fn parse_args() -> impl Parser<Output = ExpressionNodes> {
        seq3(
            back_guarded_expression_node_p(),
            keyword(Keyword::To),
            guarded_expression_node_p().or_syntax_error("Expected: expression"),
            |l, _, r| vec![l, r],
        )
    }
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.is_empty() {
            Ok(())
        } else if args.len() == 2 {
            args.require_integer_argument(0)?;
            args.require_integer_argument(1)
        } else {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::utils::VariantCasts;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        if interpreter.context().variables().len() > 0 {
            let start_row = interpreter.context()[0].to_positive_int()?;
            let end_row = interpreter.context()[1].to_positive_int()?;
            if start_row >= end_row {
                Err(QError::IllegalFunctionCall)
            } else {
                // we have args
                interpreter.screen_mut().set_view_print(start_row, end_row);
                Ok(())
            }
        } else {
            // reset full view
            interpreter.screen_mut().reset_view_print();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::built_ins::BuiltInSub;
    use crate::parser::test_utils::{parse, DemandSingleStatement, ExpressionNodeLiteralFactory};
    use crate::parser::Statement;

    #[test]
    fn parse_no_args() {
        let input = "VIEW PRINT";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::ViewPrint, vec![])
        );
    }

    #[test]
    fn parse_args() {
        let input = "VIEW PRINT 1 TO 20";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::ViewPrint,
                vec![1.as_lit_expr(1, 12), 20.as_lit_expr(1, 17)]
            )
        );
    }
}
