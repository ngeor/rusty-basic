use crate::assert_prints;

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

    assert_prints!(input, "Hearts   ", "1");
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

    assert_prints!(input, "1234", "CZ");
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
    assert_prints!(input, "1234 A");
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
    assert_prints!(input, "1234 A");
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
    assert_prints!(input, "Hello", "42");
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
    assert_prints!(input, "original value ", "modified value ");
}

#[test]
fn test_modify_in_sub() {
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
    assert_prints!(input, "2", "1");
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
    assert_prints!(input, "ab  cd  ");
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
    assert_prints!(input, "1234");
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
    assert_prints!(input, "9876");
}
