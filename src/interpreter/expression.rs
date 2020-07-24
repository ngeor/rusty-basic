#[cfg(test)]
mod tests {
    use crate::assert_has_variable;
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::interpreter::test_utils::*;
    use crate::linter::LinterError;

    #[test]
    fn test_literals() {
        assert_has_variable!(interpret("X = 3.14"), "X!", 3.14_f32);
        assert_has_variable!(interpret("X# = 3.14"), "X#", 3.14);
        assert_has_variable!(interpret("X$ = \"hello\""), "X$", "hello");
        assert_has_variable!(interpret("X% = 42"), "X%", 42);
        assert_has_variable!(interpret("X& = 42"), "X&", 42_i64);
    }

    mod binary_plus {
        use super::*;

        #[test]
        fn test_left_float() {
            assert_has_variable!(interpret("X = 1.1 + 2.1"), "X!", 3.2_f32);
            assert_has_variable!(interpret("X = 1.1 + 2.1#"), "X!", 3.2_f32);
            assert_has_variable!(interpret("X = 1.1 + 2"), "X!", 3.1_f32);
            assert_linter_err!("X = 1.1 + \"hello\"", LinterError::TypeMismatch, 1, 11);
        }

        #[test]
        fn test_left_double() {
            assert_has_variable!(interpret("X# = 1.1# + 2.1"), "X#", 3.2_f64);
            assert_has_variable!(interpret("X# = 1.1 + 2.1#"), "X#", 3.2_f64);
            assert_has_variable!(interpret("X# = 1.1# + 2"), "X#", 3.1_f64);
            assert_linter_err!("X = 1.1# + \"hello\"", LinterError::TypeMismatch, 1, 12);
        }

        #[test]
        fn test_left_string() {
            assert_has_variable!(interpret(r#"X$ = "hello" + " hi""#), "X$", "hello hi");
            assert_linter_err!("X$ = \"hello\" + 1", LinterError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" + 1.1", LinterError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" + 1.1#", LinterError::TypeMismatch, 1, 16);
        }

        #[test]
        fn test_left_integer() {
            assert_has_variable!(interpret("X% = 1 + 2.1"), "X%", 3);
            assert_has_variable!(interpret("X% = 1 + 2.5"), "X%", 4);
            assert_has_variable!(interpret("X% = 1 + 2.1#"), "X%", 3);
            assert_has_variable!(interpret("X% = 1 + 2"), "X%", 3);
            assert_linter_err!("X% = 1 + \"hello\"", LinterError::TypeMismatch, 1, 10);
        }

        #[test]
        fn test_left_long() {
            assert_has_variable!(interpret("X& = 1 + 2.1"), "X&", 3_i64);
            assert_has_variable!(interpret("X& = 1 + 2.5"), "X&", 4_i64);
            assert_has_variable!(interpret("X& = 1 + 2.1#"), "X&", 3_i64);
            assert_has_variable!(interpret("X& = 1 + 2"), "X&", 3_i64);
            assert_linter_err!("X& = 1 + \"hello\"", LinterError::TypeMismatch, 1, 10);
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
            assert_linter_err!("X = 1.1 - \"hello\"", LinterError::TypeMismatch, 1, 11);
        }

        #[test]
        fn test_left_double() {
            assert_has_variable!(interpret("X# = 5.4# - 2.1"), "X#", 3.3_f64);
            assert_has_variable!(interpret("X# = 5.4 - 2.1#"), "X#", 3.3_f64);
            assert_has_variable!(interpret("X# = 5.1# - 2"), "X#", 3.1_f64);
            assert_linter_err!("X = 1.1# - \"hello\"", LinterError::TypeMismatch, 1, 12);
        }

        #[test]
        fn test_left_string() {
            assert_linter_err!("X$ = \"hello\" - \"hi\"", LinterError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" - 1", LinterError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" - 1.1", LinterError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" - 1.1#", LinterError::TypeMismatch, 1, 16);
        }

        #[test]
        fn test_left_integer() {
            assert_has_variable!(interpret("X% = 5 - 2.1"), "X%", 3);
            assert_has_variable!(interpret("X% = 6 - 2.5"), "X%", 4);
            assert_has_variable!(interpret("X% = 5 - 2.1#"), "X%", 3);
            assert_has_variable!(interpret("X% = 5 - 2"), "X%", 3);
            assert_linter_err!("X$ = 1 - \"hello\"", LinterError::TypeMismatch, 1, 10);
        }

        #[test]
        fn test_left_long() {
            assert_has_variable!(interpret("X& = 5 - 2.1"), "X&", 3_i64);
            assert_has_variable!(interpret("X& = 6 - 2.5"), "X&", 4_i64);
            assert_has_variable!(interpret("X& = 5 - 2.1#"), "X&", 3_i64);
            assert_has_variable!(interpret("X& = 5 - 2"), "X&", 3_i64);
            assert_linter_err!("X& = 1 - \"hello\"", LinterError::TypeMismatch, 1, 10);
        }
    }

    mod multiply {
        use super::*;

        #[test]
        fn test_multiply() {
            assert_prints!("PRINT 6 * 7", "42");
        }

        #[test]
        fn test_multiply_string_linter_err() {
            assert_linter_err!(r#"PRINT "hello" * 5"#, LinterError::TypeMismatch, 1, 17);
        }
    }

    mod divide {
        use super::*;

        #[test]
        fn test_divide() {
            assert_prints!("PRINT 10 / 2", "5");
        }

        #[test]
        fn test_divide_string_linter_err() {
            assert_linter_err!(r#"PRINT "hello" / 5"#, LinterError::TypeMismatch, 1, 17);
        }
    }

    mod unary_minus {
        use super::*;

        #[test]
        fn test_unary_minus_float() {
            assert_has_variable!(interpret("X = -1.1"), "X!", -1.1_f32);
            assert_has_variable!(interpret("X = -1.1#"), "X!", -1.1_f32);
            assert_has_variable!(interpret("X = -1"), "X!", -1.0_f32);
            assert_linter_err!("X = -\"hello\"", LinterError::TypeMismatch, 1, 6);
        }

        #[test]
        fn test_unary_minus_integer() {
            assert_has_variable!(interpret("X% = -1.1"), "X%", -1);
            assert_has_variable!(interpret("X% = -1.1#"), "X%", -1);
            assert_has_variable!(interpret("X% = -1"), "X%", -1);
            assert_linter_err!("X% = -\"hello\"", LinterError::TypeMismatch, 1, 7);
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
            assert_linter_err!("X = NOT \"hello\"", LinterError::TypeMismatch, 1, 9);
        }

        #[test]
        fn test_unary_not_integer() {
            assert_has_variable!(interpret("X% = NOT 1"), "X%", -2);
            assert_has_variable!(interpret("X% = NOT 0"), "X%", -1);
            assert_has_variable!(interpret("X% = NOT -1"), "X%", 0);
            assert_has_variable!(interpret("X% = NOT -2"), "X%", 1);
            assert_linter_err!("X% = NOT \"hello\"", LinterError::TypeMismatch, 1, 10);
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
            if interpret(program).stdlib.output.len() > 0 {
                panic!(format!(
                    "Expected condition to be true but was false: {}",
                    $condition
                ))
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
            if interpret(program).stdlib.output.len() > 0 {
                panic!(format!(
                    "Expected condition to be false but was true: {}",
                    $condition
                ))
            }
        };
    }

    macro_rules! assert_condition_err {
        ($condition:expr, $col:expr) => {
            let program = format!(
                "
            IF {} THEN
                PRINT \"hi\"
            END IF
            ",
                $condition
            );
            assert_linter_err!(program, LinterError::TypeMismatch, 2, $col);
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

            assert_condition_err!("9.1 < \"hello\"", 22);

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

            assert_condition_err!("9.1# < \"hello\"", 23);

            assert_condition_false!("9.1# < 2.1#");
            assert_condition_false!("9.1# < 9.1#");
            assert_condition!("9.1# < 19.1#");
        }

        #[test]
        fn test_left_string() {
            assert_condition_err!("\"hello\" < 3.14", 26);
            assert_condition_err!("\"hello\" < 3", 26);
            assert_condition_err!("\"hello\" < 3.14#", 26);

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

            assert_condition_err!("9 < \"hello\"", 20);

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

            assert_condition_err!("9.1 <= \"hello\"", 23);

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

            assert_condition_err!("9.1# <= \"hello\"", 24);

            assert_condition_false!("9.1# <= 2.1#");
            assert_condition!("9.1# <= 9.1#");
            assert_condition!("9.1# <= 19.1#");
        }

        #[test]
        fn test_left_string() {
            assert_condition_err!("\"hello\" <= 3.14", 27);
            assert_condition_err!("\"hello\" <= 3", 27);
            assert_condition_err!("\"hello\" <= 3.14#", 27);

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

            assert_condition_err!("9 <= \"hello\"", 21);

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
    }

    mod geq {
        use super::*;

        #[test]
        fn test_greater_or_equal() {
            assert_condition!("1 >= 1");
            assert_condition!("2 >= 1");
            assert_condition_false!("1 >= 2");
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
        fn test_and_linter_error_for_strings() {
            assert_linter_err!(r#"PRINT 1 AND "hello""#, LinterError::TypeMismatch, 1, 13);
            assert_linter_err!(r#"PRINT "hello" AND 1"#, LinterError::TypeMismatch, 1, 19);
            assert_linter_err!(
                r#"PRINT "hello" AND "bye""#,
                LinterError::TypeMismatch,
                1,
                19
            );
        }

        #[test]
        fn test_and_linter_error_for_file_handle() {
            assert_linter_err!(r#"PRINT 1 AND #1"#, LinterError::TypeMismatch, 1, 13);
            assert_linter_err!(r#"PRINT #1 AND 1"#, LinterError::TypeMismatch, 1, 7);
            assert_linter_err!(r#"PRINT #1 AND #1"#, LinterError::TypeMismatch, 1, 7);
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
}
