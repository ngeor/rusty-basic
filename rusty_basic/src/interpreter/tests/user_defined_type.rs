use crate::assert_prints_exact;
use crate::interpreter::interpreter_trait::InterpreterTrait;

#[test]
fn test_user_defined_type_card() {
    let input = r#"
    TYPE Card
        Suit AS STRING * 9
        Value AS INTEGER
    END TYPE

    DIM C AS Card
    C.Suit = "Hearts"
    C.Value = 1

    PRINT C.Suit
    PRINT C.Value
    "#;

    assert_prints_exact!(input, "Hearts   ", " 1 ", "");
}

#[test]
fn test_user_defined_type_nested() {
    let input = r#"
    TYPE PostCode
        Prefix AS INTEGER
        Suffix AS STRING * 2
    END TYPE

    TYPE Address
        Street AS STRING * 100
        PostCode AS PostCode
    END TYPE

    DIM A AS Address
    A.PostCode.Prefix = 1234
    A.PostCode.Suffix =  "CZ"

    PRINT A.PostCode.Prefix
    PRINT A.PostCode.Suffix
    "#;

    assert_prints_exact!(input, " 1234 ", "CZ", "");
}

#[test]
fn test_truncate_string_at_declared_length() {
    let input = r#"
    TYPE Address ' A basic address type
        ' post code
        PostCode AS STRING * 6 ' comment here
        ' comment here too
    END TYPE

    DIM a AS Address
    a.PostCode = "1234 AZ"
    PRINT a.PostCode
    "#;
    assert_prints_exact!(input, "1234 A", "");
}

#[test]
fn test_string_const_length() {
    let input = r#"
    CONST L = 6
    TYPE Address
        PostCode AS STRING * L
    END TYPE

    DIM a AS Address
    a.PostCode = "1234 AZ"
    PRINT a.PostCode
    "#;
    assert_prints_exact!(input, "1234 A", "");
}

#[test]
fn test_assign() {
    let input = r#"
    TYPE Address
        Street AS STRING * 5
        HouseNumber AS INTEGER
    END TYPE
    DIM a AS Address
    DIM b AS Address
    a.Street = "Hello"
    a.HouseNumber = 42
    b = a
    PRINT b.Street
    PRINT b.HouseNumber
    "#;
    assert_prints_exact!(input, "Hello", " 42 ", "");
}

#[test]
fn test_assign_is_by_val() {
    let input = r#"
    TYPE Address
        Street AS STRING * 15
        HouseNumber AS INTEGER
    END TYPE
    DIM a AS Address
    DIM b AS Address

    a.Street = "original value"
    a.HouseNumber = 42

    b = a
    b.Street = "modified value"

    PRINT a.Street
    PRINT b.Street
    "#;
    assert_prints_exact!(input, "original value ", "modified value ", "");
}

#[test]
fn test_modify_members_in_sub() {
    let input = r#"
    TYPE LPARAM
        LoWord AS INTEGER
        HiWord AS INTEGER
    END TYPE

    DECLARE SUB Swap.LParam(x AS LPARAM)

    DIM p AS LPARAM
    p.LoWord = 1
    p.HiWord = 2
    Swap.LParam p
    PRINT p.LoWord
    PRINT p.HiWord

    SUB Swap.LParam(x AS LPARAM)
        p = x.LoWord
        x.LoWord = x.HiWord
        x.HiWord = p
    END SUB
    "#;
    assert_prints_exact!(input, " 2 ", " 1 ", "");
}

mod modify_in_sub {
    use super::*;

    #[test]
    fn create_new_local_value_and_copy_to_param() {
        let input = r#"
        TYPE PostCode
            Prefix AS STRING * 4
        END TYPE

        TYPE Address
            PostCode AS PostCode
        END TYPE

        TYPE Person
            Address AS Address
        END TYPE

        DIM a AS Person
        CreateNewLocalValueAndCopyToParam a
        PRINT a.Address.PostCode.Prefix

        SUB CreateNewLocalValueAndCopyToParam(b AS Person)
            DIM c AS Person
            c.Address.PostCode.Prefix = "1234 rest should be trimmed"
            b = c
        END SUB
        "#;
        assert_prints_exact!(input, "1234", "");
    }

    #[test]
    fn modify_person() {
        let input = r#"
        TYPE PostCode
            Prefix AS STRING * 4
        END TYPE

        TYPE Address
            PostCode AS PostCode
        END TYPE

        TYPE Person
            Address AS Address
        END TYPE

        DIM a AS Person
        ModifyPerson a
        PRINT a.Address.PostCode.Prefix

        SUB ModifyPerson(b AS Person)
            b.Address.PostCode.Prefix = "1234 rest should be trimmed"
        END SUB
        "#;
        assert_prints_exact!(input, "1234", "");
    }

    #[test]
    fn modify_address() {
        let input = r#"
        TYPE PostCode
            Prefix AS STRING * 4
        END TYPE

        TYPE Address
            PostCode AS PostCode
        END TYPE

        TYPE Person
            Address AS Address
        END TYPE

        DIM a AS Person
        ModifyAddress a.Address
        PRINT a.Address.PostCode.Prefix

        SUB ModifyAddress(b AS Address)
            b.PostCode.Prefix = "1234 rest should be trimmed"
        END SUB
        "#;
        assert_prints_exact!(input, "1234", "");
    }

    #[test]
    fn modify_postcode() {
        let input = r#"
        TYPE PostCode
            Prefix AS STRING * 4
        END TYPE

        TYPE Address
            PostCode AS PostCode
        END TYPE

        TYPE Person
            Address AS Address
        END TYPE

        DIM a AS Person
        ModifyPostCode a.Address.PostCode
        PRINT a.Address.PostCode.Prefix

        SUB ModifyPostCode(b AS PostCode)
            b.Prefix = "1234 rest should be trimmed"
            PRINT b.Prefix
        END SUB
        "#;
        assert_prints_exact!(input, "1234", "1234", "");
    }

    #[test]
    fn modify_prefix() {
        let input = r#"
        TYPE PostCode
            Prefix AS STRING * 4
        END TYPE

        TYPE Address
            PostCode AS PostCode
        END TYPE

        TYPE Person
            Address AS Address
        END TYPE

        DIM a AS Person
        ModifyPrefix a.Address.PostCode.Prefix
        PRINT a.Address.PostCode.Prefix

        SUB ModifyPrefix(b AS STRING)
            b$ = "1234 rest should be trimmed"
            PRINT b$
        END SUB
        "#;
        assert_prints_exact!(input, "1234 rest should be trimmed", "1234", "");
    }
}

#[test]
fn test_concatenate_two_strings_of_fixed_length() {
    let input = r#"
    TYPE PostCode
        Prefix AS STRING * 4
    END TYPE

    DIM A AS PostCode
    DIM B AS PostCode
    A.Prefix = "ab"
    B.Prefix = "cd"
    C$ = A.Prefix + B.Prefix
    PRINT C$
    "#;
    assert_prints_exact!(input, "ab  cd  ", "");
}

#[test]
fn test_assign_element_type_assign_qualified_use_unqualified() {
    let input = r#"
    TYPE PostCode
        Prefix AS STRING * 4
    END TYPE
    DIM p AS PostCode
    p.Prefix$ = "1234"
    PRINT p.Prefix
    "#;
    assert_prints_exact!(input, "1234", "");
}

#[test]
fn test_assign_element_type_assign_unqualified_use_qualified() {
    let input = r#"
    TYPE PostCode
        Prefix AS STRING * 4
    END TYPE
    DIM p AS PostCode
    p.Prefix = "9876"
    PRINT p.Prefix$
    "#;
    assert_prints_exact!(input, "9876", "");
}

#[test]
fn test_max_length_string() {
    let input = r#"
    TYPE PostCode
        Prefix AS STRING * 32767
    END TYPE
    DIM p AS PostCode
    PRINT LEN(p)
    "#;
    assert_prints_exact!(input, " 32767 ", "");
}
