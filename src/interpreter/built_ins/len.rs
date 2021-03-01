// LEN(str_expr$) -> number of characters in string
// LEN(variable) -> number of bytes required to store a variable
use super::*;
use crate::common::Locatable;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
    let v: &Variant = &interpreter.context()[0];
    let len: i32 = match v {
        Variant::VSingle(_) => 4,
        Variant::VDouble(_) => 8,
        Variant::VString(v) => v.len().try_into().unwrap(),
        Variant::VInteger(_) => 2,
        Variant::VLong(_) => 4,
        Variant::VUserDefined(user_defined_value) => {
            let user_defined_type = interpreter
                .user_defined_types()
                .get(user_defined_value.type_name())
                .unwrap();
            let sum: u32 =
                len_of_user_defined_type(user_defined_type, interpreter.user_defined_types());
            sum as i32
        }
        Variant::VArray(_) => {
            return Err(QError::ArgumentTypeMismatch).with_err_no_pos();
        }
    };
    interpreter
        .context_mut()
        .set_variable(BuiltInFunction::Len.into(), len.into());
    Ok(())
}

fn len_of_user_defined_type(user_defined_type: &UserDefinedType, types: &UserDefinedTypes) -> u32 {
    let mut sum: u32 = 0;
    for Locatable { element, .. } in user_defined_type.elements() {
        sum += match element.element_type() {
            ElementType::Single => 4,
            ElementType::Double => 8,
            ElementType::Integer => 2,
            ElementType::Long => 4,
            ElementType::FixedLengthString(_, l) => *l as u32,
            ElementType::UserDefined(Locatable {
                element: type_name, ..
            }) => len_of_user_defined_type(types.get(type_name).expect("type not found"), types),
        };
    }
    sum
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
}
