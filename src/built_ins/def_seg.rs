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
        keyword_pair_p(Keyword::Def, Keyword::Seg)
            .and_opt(equal_sign_and_expression())
            .keep_right()
            .map(|opt_arg| {
                let args: ExpressionNodes = match opt_arg {
                    Some(arg) => vec![arg],
                    _ => vec![],
                };
                Statement::BuiltInSubCall(BuiltInSub::DefSeg, args)
            })
    }

    fn equal_sign_and_expression<R>() -> impl Parser<R, Output = ExpressionNode>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        item_p('=')
            .surrounded_by_opt_ws()
            .and_demand(expression::expression_node_p())
            .keep_right()
    }
}

pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.is_empty() {
            Ok(())
        } else {
            args.require_one_numeric_argument()
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::variant::QBNumberCast;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        if interpreter.context().variables().len() == 0 {
            todo!()
        } else {
            let address: i64 = interpreter.context()[0].try_cast()?;
            if address >= 0 && address <= 65535 {
                todo!()
            } else {
                Err(QError::Overflow)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_linter_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::QError;
    use crate::parser::test_utils::{parse, DemandSingleStatement, ExpressionNodeLiteralFactory};
    use crate::parser::*;

    #[test]
    fn parse_no_items_is_allowed() {
        let input = "DEF SEG";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::DefSeg, vec![])
        );
    }

    #[test]
    fn parse_one_item() {
        let input = "DEF SEG = 42";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::DefSeg, vec![42.as_lit_expr(1, 11)])
        );
    }

    #[test]
    fn address_cannot_be_string() {
        let input = "DEF SEG = A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn address_cannot_exceed_65535() {
        let input = "DEF SEG = 65536";
        assert_interpreter_err!(input, QError::Overflow, 1, 1);
    }
}
