use crate::assert_has_variable;
use crate::assert_prints;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::test_utils::*;

mod binary_plus {
    use super::*;

    #[test]
    fn test_left_float() {
        assert_has_variable!(interpret("X = 1.1 + 2.1"), "X!", 3.2_f32);
        assert_has_variable!(interpret("X = 1.1 + 2.1#"), "X!", 3.2_f32);
        assert_has_variable!(interpret("X = 1.1 + 2"), "X!", 3.1_f32);
    }

    #[test]
    fn test_left_double() {
        assert_has_variable!(interpret("X# = 1.1# + 2.1"), "X#", 3.2_f64);
        assert_has_variable!(interpret("X# = 1.1 + 2.1#"), "X#", 3.2_f64);
        assert_has_variable!(interpret("X# = 1.1# + 2"), "X#", 3.1_f64);
    }

    #[test]
    fn test_left_string() {
        assert_has_variable!(interpret(r#"X$ = "hello" + " hi""#), "X$", "hello hi");
    }

    #[test]
    fn test_left_integer() {
        assert_has_variable!(interpret("X% = 1 + 2.1"), "X%", 3);
        assert_has_variable!(interpret("X% = 1 + 2.5"), "X%", 4);
        assert_has_variable!(interpret("X% = 1 + 2.1#"), "X%", 3);
        assert_has_variable!(interpret("X% = 1 + 2"), "X%", 3);
    }

    #[test]
    fn test_left_long() {
        assert_has_variable!(interpret("X& = 1 + 2.1"), "X&", 3_i64);
        assert_has_variable!(interpret("X& = 1 + 2.5"), "X&", 4_i64);
        assert_has_variable!(interpret("X& = 1 + 2.1#"), "X&", 3_i64);
        assert_has_variable!(interpret("X& = 1 + 2"), "X&", 3_i64);
    }

    #[test]
    fn test_function_call_plus_literal() {
        let program = r#"
        DECLARE FUNCTION Sum(A, B)

        PRINT Sum(1, 2) + 1

        FUNCTION Sum(A, B)
            Sum = A + B
        END FUNCTION
        "#;
        assert_prints!(program, "4");
    }

    #[test]
    fn test_literal_plus_function_call() {
        let program = r#"
        DECLARE FUNCTION Sum(A, B)

        PRINT 1 + Sum(1, 2)

        FUNCTION Sum(A, B)
            Sum = A + B
        END FUNCTION
        "#;
        assert_prints!(program, "4");
    }
}

mod binary_minus {
    use super::*;

    #[test]
    fn test_left_float() {
        assert_has_variable!(interpret("X = 5.4 - 2.1"), "X!", 3.3_f32);
        assert_has_variable!(interpret("X = 5.4 - 2.1#"), "X!", 3.3_f32);
        assert_has_variable!(interpret("X = 5.1 - 2"), "X!", 3.1_f32);
    }

    #[test]
    fn test_left_double() {
        assert_has_variable!(interpret("X# = 5.4# - 2.1"), "X#", 3.3_f64);
        assert_has_variable!(interpret("X# = 5.4 - 2.1#"), "X#", 3.3_f64);
        assert_has_variable!(interpret("X# = 5.1# - 2"), "X#", 3.1_f64);
    }

    #[test]
    fn test_left_integer() {
        assert_has_variable!(interpret("X% = 5 - 2.1"), "X%", 3);
        assert_has_variable!(interpret("X% = 6 - 2.5"), "X%", 4);
        assert_has_variable!(interpret("X% = 5 - 2.1#"), "X%", 3);
        assert_has_variable!(interpret("X% = 5 - 2"), "X%", 3);
    }

    #[test]
    fn test_left_long() {
        assert_has_variable!(interpret("X& = 5 - 2.1"), "X&", 3_i64);
        assert_has_variable!(interpret("X& = 6 - 2.5"), "X&", 4_i64);
        assert_has_variable!(interpret("X& = 5 - 2.1#"), "X&", 3_i64);
        assert_has_variable!(interpret("X& = 5 - 2"), "X&", 3_i64);
    }

    #[test]
    fn plus_minus() {
        assert_prints!("PRINT 4 - 2 + 1", "3");
    }

    #[test]
    fn double_minus() {
        assert_prints!("PRINT 4 - 1 - 1", "2");
    }
}

mod multiply {
    use super::*;

    #[test]
    fn test_multiply() {
        assert_prints!("PRINT 6 * 7", "42");
    }

    #[test]
    fn test_multiply_variable_with_literal() {
        let input = r#"
        DIM A
        A = 7
        PRINT A * 3
        "#;
        assert_prints!(input, "21");
    }

    #[test]
    fn test_multiply_literal_with_variable() {
        let input = r#"
        DIM A
        A = 5
        PRINT 3 * A
        "#;
        assert_prints!(input, "15");
    }
}

mod divide {
    use super::*;

    #[test]
    fn test_divide() {
        assert_prints!("PRINT 10 / 2", "5");
    }

    #[test]
    fn test_double_divide() {
        assert_prints!("PRINT 12 / 2 / 3", "2");
    }
}

mod modulo {
    use super::*;

    #[test]
    fn modulo_rounding() {
        assert_prints!("PRINT 19 MOD 6.7", "5");
    }

    #[test]
    fn modulo_priority() {
        assert_prints!("PRINT 5 MOD 4", "1");
        assert_prints!("PRINT 5 MOD 3 + 1", "3");
        assert_prints!("PRINT 1 + 5 MOD 3", "3");
        assert_prints!("PRINT 10 MOD 2", "0");
        assert_prints!("PRINT 10 MOD 2 * 2", "2");
    }
}

mod unary_minus {
    use super::*;

    #[test]
    fn test_unary_minus_float() {
        assert_has_variable!(interpret("X = -1.1"), "X!", -1.1_f32);
        assert_has_variable!(interpret("X = -1.1#"), "X!", -1.1_f32);
        assert_has_variable!(interpret("X = -1"), "X!", -1.0_f32);
    }

    #[test]
    fn test_unary_minus_integer() {
        assert_has_variable!(interpret("X% = -1.1"), "X%", -1);
        assert_has_variable!(interpret("X% = -1.1#"), "X%", -1);
        assert_has_variable!(interpret("X% = -1"), "X%", -1);
    }
}

mod unary_not {
    use super::*;

    #[test]
    fn test_unary_not_float() {
        assert_has_variable!(interpret("X = NOT 3.14"), "X!", -4.0_f32);
        assert_has_variable!(interpret("X = NOT 3.5#"), "X!", -5.0_f32);
        assert_has_variable!(interpret("X = NOT -1.1"), "X!", 0.0_f32);
        assert_has_variable!(interpret("X = NOT -1.5"), "X!", 1.0_f32);
    }

    #[test]
    fn test_unary_not_integer() {
        assert_has_variable!(interpret("X% = NOT 1"), "X%", -2);
        assert_has_variable!(interpret("X% = NOT 0"), "X%", -1);
        assert_has_variable!(interpret("X% = NOT -1"), "X%", 0);
        assert_has_variable!(interpret("X% = NOT -2"), "X%", 1);
    }
}

macro_rules! assert_condition {
    ($condition:expr) => {
        let program = format!(
            "
        IF {} THEN
        ELSE
            PRINT \"hi\"
        END IF
        ",
            $condition
        );
        if interpret(&program).stdout().output().len() > 0 {
            panic!(
                "Expected: condition to be true but was false: {}",
                $condition
            )
        }
    };
}

macro_rules! assert_condition_false {
    ($condition:expr) => {
        let program = format!(
            "
        IF {} THEN
            PRINT \"hi\"
        END IF
        ",
            $condition
        );
        if interpret(&program).stdout().output().len() > 0 {
            panic!(
                "Expected: condition to be false but was true: {}",
                $condition
            )
        }
    };
}

mod less {
    use super::*;

    #[test]
    fn test_left_float() {
        assert_condition_false!("9.1 < 2.1");
        assert_condition_false!("9.1 < 9.1");
        assert_condition!("9.1 < 19.1");

        assert_condition_false!("9.1 < 2");
        assert_condition_false!("9.1 < 9");
        assert_condition!("9.1 < 19");

        assert_condition_false!("9.1 < 2.1#");
        assert_condition_false!("9.1 < 9.1#");
        assert_condition!("9.1 < 19.1#");
    }

    #[test]
    fn test_left_double() {
        assert_condition_false!("9.1# < 2.1");
        assert_condition_false!("9.1# < 9.1");
        assert_condition!("9.1# < 19.1");

        assert_condition_false!("9.1# < 2");
        assert_condition_false!("9.1# < 9");
        assert_condition!("9.1# < 19");

        assert_condition_false!("9.1# < 2.1#");
        assert_condition_false!("9.1# < 9.1#");
        assert_condition!("9.1# < 19.1#");
    }

    #[test]
    fn test_left_string() {
        assert_condition_false!("\"def\" < \"abc\"");
        assert_condition_false!("\"def\" < \"def\"");
        assert_condition!("\"def\" < \"xyz\"");
    }

    #[test]
    fn test_left_integer() {
        assert_condition_false!("9 < 2.1");
        assert_condition_false!("9 < 8.9");
        assert_condition_false!("9 < 9.0");
        assert_condition!("9 < 9.1");
        assert_condition!("9 < 19.1");

        assert_condition_false!("9 < 2");
        assert_condition_false!("9 < 9");
        assert_condition!("9 < 19");

        assert_condition_false!("9 < 2.1#");
        assert_condition!("9 < 9.1#");
        assert_condition!("9 < 19.1#");
    }
}

mod lte {
    use super::*;

    #[test]
    fn test_left_float() {
        assert_condition_false!("9.1 <= 2.1");
        assert_condition!("9.1 <= 9.1");
        assert_condition!("9.1 <= 19.1");

        assert_condition_false!("9.1 <= 2");
        assert_condition_false!("9.1 <= 9");
        assert_condition!("9.1 <= 19");

        assert_condition_false!("9.1 <= 2.1#");
        assert_condition!("9.1 <= 9.1#");
        assert_condition!("9.1 <= 19.1#");
    }

    #[test]
    fn test_left_double() {
        assert_condition_false!("9.1# <= 2.1");
        assert_condition!("9.1# <= 9.1");
        assert_condition!("9.1# <= 19.1");

        assert_condition_false!("9.1# <= 2");
        assert_condition_false!("9.1# <= 9");
        assert_condition!("9.1# <= 19");

        assert_condition_false!("9.1# <= 2.1#");
        assert_condition!("9.1# <= 9.1#");
        assert_condition!("9.1# <= 19.1#");
    }

    #[test]
    fn test_left_string() {
        assert_condition_false!("\"def\" <= \"abc\"");
        assert_condition!("\"def\" <= \"def\"");
        assert_condition!("\"def\" <= \"xyz\"");
    }

    #[test]
    fn test_left_integer() {
        assert_condition_false!("9 <= 2.1");
        assert_condition_false!("9 <= 8.9");
        assert_condition!("9 <= 9.0");
        assert_condition!("9 <= 9.1");
        assert_condition!("9 <= 19.1");

        assert_condition_false!("9 <= 2");
        assert_condition!("9 <= 9");
        assert_condition!("9 <= 19");

        assert_condition_false!("9 <= 2.1#");
        assert_condition!("9 <= 9.1#");
        assert_condition!("9 <= 19.1#");
    }
}

mod eq {
    use super::*;

    #[test]
    fn test_equality() {
        assert_condition!("1 = 1");
        assert_condition_false!("1 = 2");
    }

    #[test]
    fn test_equality_string() {
        assert_condition!(r#""ABC" = "ABC""#);
        assert_condition_false!(r#""ABC" = "DEF""#);
    }
}

mod geq {
    use super::*;

    #[test]
    fn test_greater_or_equal() {
        assert_condition!("1 >= 1");
        assert_condition!("2 >= 1");
        assert_condition_false!("1 >= 2");
    }

    #[test]
    fn test_greater_or_equal_string() {
        assert_condition!(r#""DEF" >= "ABC""#);
        assert_condition!(r#""DEF" >= "DEF""#);
        assert_condition_false!(r#""ABC" >= "DEF""#);
    }
}

mod gt {
    use super::*;

    #[test]
    fn test_greater() {
        assert_condition!("2 > 1");
        assert_condition_false!("2 > 2");
        assert_condition_false!("1 > 2");
    }
}

mod ne {
    use super::*;

    #[test]
    fn test_not_equals() {
        assert_condition!("2 <> 1");
        assert_condition_false!("1 <> 1");
    }
}

mod and {
    use super::*;

    #[test]
    fn test_and_positive_ones_zeroes() {
        assert_condition!("1 AND 1");
        assert_condition_false!("1 AND 0");
        assert_condition_false!("0 AND 1");
        assert_condition_false!("0 AND 0");
    }

    #[test]
    fn test_and_negative_ones_zeroes() {
        assert_condition!("-1 AND -1");
        assert_condition_false!("-1 AND 0");
        assert_condition_false!("0 AND -1");
        assert_condition_false!("0 AND 0");
    }

    #[test]
    fn test_and_binary_arithmetic_positive_positive() {
        assert_prints!("PRINT 5 AND 2", "0");
        assert_prints!("PRINT 5 AND 1", "1");
        assert_prints!("PRINT 1 AND 1", "1");
        assert_prints!("PRINT 7 AND 1", "1");
        assert_prints!("PRINT 7 AND 2", "2");
        assert_prints!("PRINT 7 AND 6", "6");
        assert_prints!("PRINT 6 AND 7", "6");
    }

    #[test]
    fn test_and_binary_arithmetic_positive_negative() {
        assert_prints!("PRINT 5 AND -2", "4");
        assert_prints!("PRINT -5 AND 2", "2");
        assert_prints!("PRINT 5 AND -1", "5");
        assert_prints!("PRINT -5 AND 1", "1");
    }

    #[test]
    fn test_and_binary_arithmetic_negative_negative() {
        assert_prints!("PRINT -5 AND -2", "-6");
        assert_prints!("PRINT -5 AND -1", "-5");
    }

    #[test]
    fn test_and_two_string_comparisons() {
        assert_condition!(r#" "DEF" >= "ABC" AND "DEF" < "GHI" "#);
    }
}

mod or {
    use super::*;

    #[test]
    fn test_or_positive_ones_zeroes() {
        assert_condition!("1 OR 1");
        assert_condition!("1 OR 0");
        assert_condition!("0 OR 1");
        assert_condition_false!("0 OR 0");
    }

    #[test]
    fn test_or_negative_ones_zeroes() {
        assert_condition!("-1 OR -1");
        assert_condition!("-1 OR 0");
        assert_condition!("0 OR -1");
        assert_condition_false!("0 OR 0");
    }

    #[test]
    fn test_or_binary_arithmetic() {
        assert_prints!("PRINT 1 OR 1", "1");
        assert_prints!("PRINT 1 OR 0", "1");
        assert_prints!("PRINT 1 OR 2", "3");
        assert_prints!("PRINT -1 OR -1", "-1");
        assert_prints!("PRINT -1 OR 0", "-1");
        assert_prints!("PRINT -1 OR 1", "-1");
    }

    #[test]
    fn test_or_string_comparison_and_two_string_comparisons() {
        assert_condition!(r#" "DEF" >= "ABC" AND "DEF" < "GHI" OR "XYZ" = "XYZ" "#);
    }
}

mod priority {
    use super::*;

    #[test]
    fn test_and_has_priority_over_or() {
        assert_prints!("PRINT 1 OR 1 AND 0", "1");
        assert_prints!("PRINT 1 OR (1 AND 0)", "1");
        assert_prints!("PRINT (1 OR 1) AND 0", "0");

        assert_prints!("PRINT 1 OR 0 AND 0", "1");
        assert_prints!("PRINT 1 OR (0 AND 0)", "1");
        assert_prints!("PRINT (1 OR 0) AND 0", "0");

        assert_prints!("PRINT 0 AND 0 OR 1", "1");
        assert_prints!("PRINT (0 AND 0) OR 1", "1");
        assert_prints!("PRINT 0 AND (0 OR 1)", "0");
    }

    #[test]
    fn test_relational_has_priority_over_binary() {
        assert_prints!("PRINT 1 OR 2 > 1 AND 2", "3");
        assert_prints!("PRINT 1 OR (2 > 1) AND 2", "3");
        assert_prints!("PRINT 1 OR ((2 > 1) AND 2)", "3");
        assert_prints!("PRINT (1 OR 2) > (1 AND 2)", "-1");
    }

    #[test]
    fn test_arithmetic_has_priority_over_relational() {
        assert_prints!("PRINT 1 + 1 > 2", "0");
        assert_prints!("PRINT 1 + (1 > 2)", "1");
    }

    #[test]
    fn test_arithmetic_has_priority_over_binary() {
        assert_prints!("PRINT 1 + 2 OR 1", "3");
        assert_prints!("PRINT 1 + (2 OR 1)", "4");
    }

    #[test]
    fn test_binary_not_short_circuit() {
        let program = r#"
        DECLARE FUNCTION Echo(X)

        PRINT Echo(1) OR Echo(0)

        FUNCTION Echo(X)
            PRINT X
            Echo = X
        END FUNCTION
        "#;
        assert_prints!(program, "1", "0", "1");
    }

    #[test]
    fn test_multiply_divide_have_priority_over_plus_minus() {
        assert_prints!("PRINT 2 * 3 + 4", "10");
        assert_prints!("PRINT 2 + 3 * 4", "14");
        assert_prints!("PRINT 2 * 3 - 4", "2");
        assert_prints!("PRINT 2 - 3 * 4", "-10");
        assert_prints!("PRINT 6 / 3 + 4", "6");
        assert_prints!("PRINT 2 + 8 / 4", "4");
        assert_prints!("PRINT 6 / 3 - 4", "-2");
        assert_prints!("PRINT 2 - 12 / 4", "-1");
    }
}

#[test]
fn test_dot_in_expression_variable_name() {
    let program = r#"
    my.msg$ = "hello"
    IF LEN(my.msg$) > 0 THEN
        PRINT my.msg$
    ELSE
        PRINT "adios"
    END IF
    "#;
    assert_prints!(program, "hello");

    let program = r#"
    my.msg$ = ""
    IF LEN(my.msg$) > 0 THEN
        PRINT my.msg$
    ELSE
        PRINT "bye"
    END IF
    "#;
    assert_prints!(program, "bye");
}

#[test]
fn test_hexadecimals() {
    assert_prints!("PRINT &H0", "0");
    assert_prints!("PRINT &H5", "5");
    assert_prints!("PRINT &HA", "10");
    assert_prints!("PRINT &H10", "16");

    assert_prints!("PRINT &H1A", "26");
    assert_prints!("PRINT &HB2", "178");

    assert_prints!("PRINT &H1A1", "417");
    assert_prints!("PRINT &H2BC", "700");
    assert_prints!("PRINT &H34C", "844");

    assert_prints!("PRINT &HAB4", "2740");
    assert_prints!("PRINT &HB3E", "2878");
    assert_prints!("PRINT &HC45", "3141");

    assert_prints!("PRINT &HFF", "255"); // 0000 0000 1111 1111
    assert_prints!("PRINT &H00FF", "255");
    assert_prints!("PRINT &H100", "256");

    assert_prints!("PRINT &H7FFF", "32767"); // max positive integer 0111 1111 1111 1111
    assert_prints!("PRINT &H8000", "-32768"); // min negative integer 1000 0000 0000 0000
    assert_prints!("PRINT &HFFFF", "-1"); // integer
    assert_prints!("PRINT -&HFFFF", "1");

    // long
    assert_prints!("PRINT &H10000", "65536");
    assert_prints!("PRINT &H7FFFFFFF", "2147483647");
    assert_prints!("PRINT &H000007FFFFFFF", "2147483647");
    assert_prints!("PRINT &H80000000", "-2147483648");
    assert_prints!("PRINT &HFFFFFFFF", "-1");
}

#[test]
fn test_octal() {
    assert_prints!("PRINT &O0", "0");
    assert_prints!("PRINT &O5", "5");
    assert_prints!("PRINT &O10", "8");
    assert_prints!("PRINT &O010", "8");
    assert_prints!("PRINT &O377", "255");
    assert_prints!("PRINT &O400", "256");
    assert_prints!("PRINT &O2000", "1024");
    assert_prints!("PRINT &O77777", "32767");
    assert_prints!("PRINT &O100000", "-32768");
    assert_prints!("PRINT &O177777", "-1");
    assert_prints!("PRINT &O200000", "65536");
    assert_prints!("PRINT &O17777777777", "2147483647");
    assert_prints!("PRINT &O20000000000", "-2147483648");
    assert_prints!("PRINT &O37777777777", "-1");
}
