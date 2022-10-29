use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::BuiltInFunction;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let path = interpreter
        .context()
        .variables()
        .get_arg_path(0)
        .expect("VARPTR should have a variable");
    let address = interpreter.context().calculate_varptr(path)?;
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::VarPtr, address as i32);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn global_built_in_vars() {
        let input = r#"
        DIM A AS INTEGER
        DIM B AS LONG
        DIM C AS SINGLE
        DIM D AS DOUBLE
        PRINT VARPTR(A)
        PRINT VARPTR(B)
        PRINT VARPTR(C)
        PRINT VARPTR(D)
        "#;
        assert_prints!(input, "0", "2", "6", "10");
    }

    #[test]
    fn inside_sub() {
        let input = r#"
        Hello

        SUB Hello
            DIM A AS INTEGER
            DIM B AS LONG
            PRINT VARPTR(A)
            PRINT VARPTR(B)
        END SUB
        "#;
        assert_prints!(input, "0", "2");
    }

    #[test]
    fn using_shared_variable_inside_sub() {
        let input = r#"
        DIM SHARED C AS SINGLE
        Hello

        SUB Hello
            DIM A AS INTEGER
            DIM B AS LONG
            PRINT VARPTR(A)
            PRINT VARPTR(B)
            PRINT VARPTR(C)
        END SUB
        "#;
        assert_prints!(input, "4", "6", "0");
    }

    #[test]
    fn array_elements_relative_to_array() {
        let input = r#"
        DIM A(1 TO 2)
        PRINT VARPTR(A)
        PRINT VARPTR(A(1))
        PRINT VARPTR(A(2))
        "#;
        assert_prints!(input, "0", "0", "4");
    }

    #[test]
    fn multi_dimensional_array() {
        let input = r#"
        DIM A(1 TO 3, 1 TO 4)
        PRINT VARPTR(A(2, 3))
        "#;
        assert_prints!(input, "24");
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
        PRINT VARPTR(c)
        PRINT VARPTR(c.Value)
        PRINT VARPTR(c.Suit)
        PRINT VARPTR(c.Luck)
        "#;
        assert_prints!(input, "0", "0", "2", "7");
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
        PRINT VARPTR(A.PostCode.Suffix)
        "#;
        assert_prints!(input, "24");
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
        PRINT VARPTR(A(2).PostCode.Suffix)
        "#;
        assert_prints!(input, "50");
    }
}
