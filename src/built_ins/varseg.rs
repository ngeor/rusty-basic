pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_variable()
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let path = interpreter
            .context()
            .variables()
            .get_arg_path(0)
            .expect("VARSEG should have a variable");
        let address = interpreter.context().calculate_varseg(path);
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::VarSeg, address as i32);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_no_arguments() {
        assert_linter_err!("PRINT VARSEG()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arguments() {
        assert_linter_err!("PRINT VARSEG(A, B)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn test_literal_argument() {
        assert_linter_err!("PRINT VARSEG(3)", QError::VariableRequired);
    }

    #[test]
    fn global_built_in_vars() {
        let input = r#"
        DIM A AS INTEGER
        DIM B AS LONG
        DIM C AS SINGLE
        DIM D AS DOUBLE
        PRINT VARSEG(A)
        PRINT VARSEG(B)
        PRINT VARSEG(C)
        PRINT VARSEG(D)
        "#;
        assert_prints!(input, "4096", "4096", "4096", "4096");
    }

    #[test]
    fn inside_sub() {
        let input = r#"
        Hello

        SUB Hello
            DIM A AS INTEGER
            DIM B AS LONG
            PRINT VARSEG(A)
            PRINT VARSEG(B)
        END SUB
        "#;
        assert_prints!(input, "4097", "4097"); // TODO should be 4096, 4096
    }

    #[test]
    fn using_shared_variable_inside_sub() {
        let input = r#"
        DIM SHARED C AS SINGLE
        Hello

        SUB Hello
            DIM A AS INTEGER
            DIM B AS LONG
            PRINT VARSEG(A)
            PRINT VARSEG(B)
            PRINT VARSEG(C)
        END SUB
        "#;
        assert_prints!(input, "4097", "4097", "4096");
    }

    #[test]
    fn array_elements_define_new_var_seg() {
        let input = r#"
        DIM A(1 TO 2)
        PRINT VARSEG(A)
        PRINT VARSEG(A(1))
        PRINT VARSEG(A(2))
        "#;
        assert_prints!(input, "4096", "4097", "4097");
    }

    #[test]
    fn multi_dimensional_array() {
        let input = r#"
        DIM A(1 TO 3, 1 TO 4)
        PRINT VARSEG(A(2, 3))
        "#;
        assert_prints!(input, "4097");
    }

    #[test]
    fn property_elements() {
        let input = r#"
        TYPE Card
            Value AS INTEGER
            Suit AS STRING * 5
            Luck AS INTEGER
        END TYPE
        DIM c AS Card
        PRINT VARSEG(c)
        PRINT VARSEG(c.Value)
        PRINT VARSEG(c.Suit)
        PRINT VARSEG(c.Luck)
        "#;
        assert_prints!(input, "4096", "4096", "4096", "4096");
    }

    #[test]
    fn nested_property() {
        let input = r#"
        TYPE PostCode
            Digits AS STRING * 4
            Suffix AS STRING * 2
        END TYPE

        TYPE Address
            Street AS STRING * 20
            PostCode AS PostCode
        END TYPE

        DIM A AS Address
        PRINT VARSEG(A.PostCode.Suffix)
        "#;
        assert_prints!(input, "4096");
    }

    #[test]
    fn nested_property_on_array_element() {
        let input = r#"
        TYPE PostCode
            Digits AS STRING * 4
            Suffix AS STRING * 2
        END TYPE

        TYPE Address
            Street AS STRING * 20
            PostCode AS PostCode
        END TYPE

        DIM A(1 TO 5) AS Address
        PRINT VARSEG(A(2).PostCode.Suffix)
        "#;
        assert_prints!(input, "4097");
    }

    #[test]
    fn three_arrays() {
        let input = r#"
        DIM A(1 TO 2) AS INTEGER
        DIM B(1 TO 2) AS INTEGER
        DIM C(1 TO 2) AS INTEGER
        PRINT VARSEG(A)
        PRINT VARSEG(B)
        PRINT VARSEG(C)
        PRINT VARSEG(A(1))
        PRINT VARSEG(B(1))
        PRINT VARSEG(C(1))
        "#;
        assert_prints!(input, "4096", "4096", "4096", "4097", "4098", "4099");
    }

    #[test]
    fn test_two_subs() {
        let input = r#"
        FirstSub

        SUB FirstSub
            DIM A AS INTEGER
            PRINT VARSEG(A)
            SecondSub
        END SUB

        SUB SecondSub
            DIM A AS INTEGER
            PRINT VARSEG(A)
        END SUB
        "#;
        assert_prints!(input, "4097", "4098");
    }

    #[test]
    fn recursive_sub() {
        let input = r#"
        Test 3

        SUB Test(Depth)
            DIM A AS INTEGER
            PRINT VARSEG(A)
            IF Depth > 1 THEN
                Test(Depth - 1)
            END IF
        END SUB
        "#;
        assert_prints!(input, "4097", "4098", "4099");
    }
}
