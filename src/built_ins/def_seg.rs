pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::parser::base::and_pc::AndDemandTrait;
    use crate::parser::base::parsers::{AndOptTrait, FnMapTrait, KeepRightTrait, Parser};
    use crate::parser::specific::whitespace::WhitespaceTrait;
    use crate::parser::specific::{item_p, keyword_pair_p, OrSyntaxErrorTrait};
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        keyword_pair_p(Keyword::Def, Keyword::Seg)
            .and_opt(equal_sign_and_expression())
            .keep_right()
            .fn_map(opt_arg_to_args)
            .fn_map(|args| Statement::BuiltInSubCall(BuiltInSub::DefSeg, args))
    }

    fn equal_sign_and_expression() -> impl Parser<Output = ExpressionNode> {
        item_p('=')
            .surrounded_by_opt_ws()
            .and_demand(
                expression::expression_node_p()
                    .or_syntax_error("Expected expression after equal sign"),
            )
            .keep_right()
    }

    fn opt_arg_to_args(opt_arg: Option<ExpressionNode>) -> ExpressionNodes {
        match opt_arg {
            Some(arg) => vec![arg],
            _ => vec![],
        }
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
            interpreter.set_def_seg(None);
            Ok(())
        } else {
            let address: i64 = interpreter.context()[0].try_cast()?;
            if address >= 0 && address <= 65535 {
                interpreter.set_def_seg(Some(address as usize));
                Ok(())
            } else {
                Err(QError::IllegalFunctionCall)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::built_ins::BuiltInSub;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
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
    fn address_cannot_be_negative() {
        let input = "DEF SEG = -1";
        assert_interpreter_err!(input, QError::IllegalFunctionCall, 1, 1);
    }

    #[test]
    fn address_cannot_exceed_65535() {
        let input = "DEF SEG = 65536";
        assert_interpreter_err!(input, QError::IllegalFunctionCall, 1, 1);
    }

    #[test]
    fn happy_flow() {
        let input = r#"
        DIM A AS INTEGER
        P = VARPTR(A)
        DEF SEG = VARSEG(A)
        POKE P, 2     ' sets the low byte of A to 2
        POKE P + 1, 1 ' sets the high byte of A to 1
        PRINT A       ' result is 2 + 1 * 256 = 258
        "#;
        assert_prints!(input, "258");
    }

    #[test]
    fn caps_lock() {
        let input = r#"
        DEF SEG = 0                     ' Turn off CapLock, NumLock and ScrollLock
        KeyFlags = PEEK(1047)
        POKE 1047, &H0
        DEF SEG
        "#;
        assert_prints!(input, "");
    }

    #[test]
    fn data_poke() {
        let input = r#"
        DEFINT A-Z
        DATA 1, 2, 3, 4
        DIM A(1 TO 2)
        DEF SEG = VARSEG(A(1))
        FOR I = 1 TO 4
            READ X
            POKE VARPTR(A(1)) + I - 1, X
        NEXT
        DEF SEG
        PRINT A(1)
        PRINT A(2)
        "#;
        assert_prints!(input, "513", "1027");
    }
}
