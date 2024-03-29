use crate::interpreter::byte_size::QByteSize;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::RuntimeError;
use rusty_parser::BuiltInFunction;
use rusty_variant::Variant;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let v: &Variant = &interpreter.context()[0];
    let len: i32 = v.byte_size() as i32;
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Len, len);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_len_string_literal() {
        let program = r#"PRINT LEN("hello")"#;
        assert_prints!(program, "5");
    }

    #[test]
    fn test_len_string_variable() {
        let program = r#"
        A$ = "hello"
        PRINT LEN(A$)
        "#;
        assert_prints!(program, "5");
    }

    #[test]
    fn test_len_float_variable() {
        let program = "
        A = 3.14
        PRINT LEN(A)
        ";
        assert_prints!(program, "4");
    }

    #[test]
    fn test_len_double_variable() {
        let program = "
        A# = 3.14
        PRINT LEN(A#)
        ";
        assert_prints!(program, "8");
    }

    #[test]
    fn test_len_integer_variable() {
        let program = "
        A% = 42
        PRINT LEN(A%)
        ";
        assert_prints!(program, "2");
    }

    #[test]
    fn test_len_long_variable() {
        let program = "
        A& = 42
        PRINT LEN(A&)
        ";
        assert_prints!(program, "4");
    }

    #[test]
    fn test_len_user_defined_type() {
        let program = "
        TYPE Card
            Value AS INTEGER
            Suit AS STRING * 9
        END TYPE
        DIM A AS Card
        PRINT LEN(A)
        ";
        assert_prints!(program, "11");
    }

    #[test]
    fn test_len_user_defined_type_nested_one_level() {
        let program = "
        TYPE PostCode
            Prefix AS STRING * 4
            Suffix AS STRING * 2
        END TYPE
        TYPE Address
            Street AS STRING * 50
            PostCode AS PostCode
        END TYPE
        DIM A AS Address
        PRINT LEN(A)
        ";
        assert_prints!(program, "56");
    }

    #[test]
    fn test_len_user_defined_type_nested_two_levels() {
        let program = "
        TYPE PostCode
            Prefix AS STRING * 4
            Suffix AS STRING * 2
        END TYPE
        TYPE Address
            Street AS STRING * 50
            PostCode AS PostCode
        END TYPE
        TYPE Person
            FullName AS STRING * 100
            Address AS Address
        END TYPE
        DIM A AS Person
        PRINT LEN(A)
        ";
        assert_prints!(program, "156");
    }

    #[test]
    fn test_len_user_defined_type_member() {
        let program = "
        TYPE PostCode
            Prefix AS STRING * 4
            Suffix AS STRING * 2
        END TYPE
        TYPE Address
            Street AS STRING * 50
            PostCode AS PostCode
        END TYPE
        TYPE Person
            FullName AS STRING * 100
            Address AS Address
        END TYPE
        DIM A AS Person
        PRINT LEN(A.Address)
        ";
        assert_prints!(program, "56");
    }

    #[test]
    fn test_fixed_length_string() {
        let program = r#"
        DIM X AS STRING * 5
        PRINT LEN(X)
        DIM Y AS STRING
        PRINT LEN(Y)
        "#;
        assert_prints!(program, "5", "0");
    }

    #[test]
    fn test_array_element() {
        let program = r#"
        DIM A(1 TO 2) AS INTEGER
        PRINT LEN(A(1))
        "#;
        assert_prints!(program, "2");
    }
}
