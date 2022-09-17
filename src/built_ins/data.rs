pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::parser::base::parsers::{AndOptTrait, FnMapTrait, KeepRightTrait, Parser};
    use crate::parser::specific::keyword_p;
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        keyword_p(Keyword::Data)
            .and_opt(expression::expression_nodes_p())
            .keep_right()
            .map(|opt_args| {
                Statement::BuiltInSubCall(BuiltInSub::Data, opt_args.unwrap_or_default())
            })
    }
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
    use crate::linter::NameContext;
    use crate::parser::{Expression, ExpressionNode, ExpressionNodes};

    pub fn lint(args: &ExpressionNodes, name_context: NameContext) -> Result<(), QErrorNode> {
        if name_context == NameContext::Global {
            args.iter().map(require_constant).collect()
        } else {
            Err(QError::IllegalInSubFunction).with_err_no_pos()
        }
    }

    fn require_constant(arg: &ExpressionNode) -> Result<(), QErrorNode> {
        match &arg.element {
            Expression::SingleLiteral(_)
            | Expression::DoubleLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::IntegerLiteral(_)
            | Expression::LongLiteral(_) => Ok(()),
            _ => Err(QError::InvalidConstant).with_err_at(arg),
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::variant::Variant;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let variables: Vec<Variant> = interpreter
            .context()
            .variables()
            .iter()
            .map(Clone::clone)
            .collect();
        for v in variables {
            interpreter.data_segment().push(v);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::built_ins::BuiltInSub;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::parser::test_utils::{parse, DemandSingleStatement, ExpressionNodeLiteralFactory};
    use crate::parser::*;

    #[test]
    fn parse_no_items_is_allowed() {
        let input = "DATA";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::Data, vec![])
        );
    }

    #[test]
    fn parse_one_item() {
        let input = "DATA 42";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::Data, vec![42.as_lit_expr(1, 6)])
        );
    }

    #[test]
    fn parse_two_items() {
        let input = r#"DATA 42, "hello""#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Data,
                vec![42.as_lit_expr(1, 6), "hello".as_lit_expr(1, 10)]
            )
        );
    }

    #[test]
    fn data_not_allowed_in_sub() {
        let input = r#"
        SUB Hello
            DATA 1, 2
        END SUB
        "#;
        assert_linter_err!(input, QError::IllegalInSubFunction);
    }

    #[test]
    fn data_not_allowed_in_function() {
        let input = r#"
        FUNCTION Hello
            DATA 1, 2
        END FUNCTION
        "#;
        assert_linter_err!(input, QError::IllegalInSubFunction);
    }

    #[test]
    fn data_must_be_constants() {
        assert_linter_err!("DATA A", QError::InvalidConstant);
    }

    #[test]
    fn data_is_moved_at_the_start_of_the_program() {
        let input = r#"
        READ A
        PRINT A
        DATA 42
        "#;
        assert_prints!(input, "42");
    }
}
