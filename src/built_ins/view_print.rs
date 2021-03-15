pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn parse<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_pair_p(Keyword::View, Keyword::Print)
            .and_opt(parse_args())
            .keep_right()
            .map(|opt_args| {
                Statement::BuiltInSubCall(BuiltInSub::ViewPrint, opt_args.unwrap_or_default())
            })
    }

    fn parse_args<R>() -> impl Parser<R, Output = ExpressionNodes>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        expression::back_guarded_expression_node_p()
            .and_demand(keyword_p(Keyword::To).or_syntax_error("Expected: TO"))
            .and_demand(
                expression::guarded_expression_node_p().or_syntax_error("Expected: expression"),
            )
            .map(|((l, _), r)| vec![l, r])
    }
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::post_linter::built_in_linter::require_integer_argument;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.is_empty() {
            Ok(())
        } else if args.len() == 2 {
            require_integer_argument(args, 0)?;
            require_integer_argument(args, 1)
        } else {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        if interpreter.context().variables().len() > 0 {
            // we have args
            todo!()
        } else {
            // reset full view
            todo!()
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
