#[cfg(test)]
mod tests {
    use crate::assert_prints;

    mod unqualified_integer_declaration {
        use super::*;

        #[test]
        fn unqualified_usage() {
            let program = "
            CONST X = 42
            PRINT X
            ";
            assert_prints!(program, "42");
        }

        #[test]
        fn qualified_usage() {
            let program = "
            CONST X = 42
            PRINT X%
            ";
            assert_prints!(program, "42");
        }
    }

    mod unqualified_single_declaration {
        use super::*;

        #[test]
        fn unqualified_usage() {
            let program = "
            CONST X = 3.14
            PRINT X
            ";
            assert_prints!(program, "3.14");
        }

        #[test]
        fn qualified_usage() {
            let program = r#"
            CONST X = 3.14
            PRINT X!
            "#;
            assert_prints!(program, "3.14");
        }
    }

    mod unqualified_double_declaration {
        use super::*;

        #[test]
        fn unqualified_usage() {
            let program = "
            CONST X = 3.14#
            PRINT X
            ";
            assert_prints!(program, "3.14");
        }
    }

    mod unqualified_string_declaration {
        use super::*;

        #[test]
        fn unqualified_usage() {
            let program = r#"
            CONST X = "hello"
            PRINT X
            "#;
            assert_prints!(program, "hello");
        }
    }

    mod qualified_single_declaration {
        use super::*;

        #[test]
        fn qualified_usage_casting_from_integer() {
            let program = "
            CONST X! = 42
            PRINT X!
            ";
            assert_prints!(program, "42");
        }

        #[test]
        fn qualified_usage_from_single_literal() {
            let program = "
            CONST X! = 3.14
            PRINT X!
            ";
            assert_prints!(program, "3.14");
        }
    }

    mod expressions {
        use super::*;

        #[test]
        fn binary_plus() {
            let program = r#"
            CONST X = 1
            CONST Y = X + 2
            PRINT Y
            "#;
            assert_prints!(program, "3");
        }

        #[test]
        fn binary_minus() {
            let program = r#"
            CONST X = 3
            CONST Y = X - 2
            PRINT Y
            "#;
            assert_prints!(program, "1");
        }

        #[test]
        fn unary_minus() {
            let program = r#"
            CONST X = 3
            CONST Y = -X
            PRINT Y
            "#;
            assert_prints!(program, "-3");
        }

        #[test]
        fn unary_not() {
            let program = r#"
            CONST TRUE = -1
            CONST FALSE = NOT TRUE
            PRINT FALSE
            "#;
            assert_prints!(program, "0");
        }
    }

    mod sub_usage {
        use super::*;

        #[test]
        fn simple_usage() {
            let program = r#"
            CONST X = 42
            DECLARE SUB Hello

            Hello

            SUB Hello
                PRINT X
            END SUB
            "#;

            assert_prints!(program, "42");
        }

        #[test]
        fn parameter_hides_const() {
            let program = r#"
            CONST X = 42
            DECLARE SUB Hello(X)

            Hello 5

            SUB Hello(X)
                PRINT X
            END SUB
            "#;
            assert_prints!(program, "5");
        }

        #[test]
        fn redefine() {
            let program = r#"
            CONST X = 42
            DECLARE SUB Hello

            Hello
            PRINT X

            SUB Hello
                PRINT X
                CONST X = 100
                PRINT X
            END SUB
            "#;
            assert_prints!(program, "42", "100", "42");
        }

        #[test]
        fn nested_sub() {
            let program = "
            CONST X = 42
            Sub1

            SUB Sub1
                CONST X = 3
                PRINT X
                Sub2
            END SUB

            SUB Sub2
                PRINT X
            END SUB
            ";
            assert_prints!(program, "3", "42");
        }
    }
}
