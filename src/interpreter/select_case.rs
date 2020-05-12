#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_linter_err;
    use crate::linter::LinterError;

    #[test]
    fn test_select_case_match_first() {
        let input = r#"
        SELECT CASE 1
            CASE 1
                PRINT "one"
            CASE 2
                PRINT "two"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["one"]);
    }

    #[test]
    fn test_select_case_match_second() {
        let input = r#"
        SELECT CASE 2
            CASE 1
                PRINT "one"
            CASE 2
                PRINT "two"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["two"]);
    }

    #[test]
    fn test_select_case_match_none() {
        let input = r#"
        SELECT CASE 3
            CASE 1
                PRINT "one"
            CASE 2
                PRINT "two"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, Vec::<String>::new());
    }

    #[test]
    fn test_select_case_match_first_only_once() {
        let input = r#"
        SELECT CASE 1
            CASE 1
                PRINT "one"
            CASE 1
                PRINT "one"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["one"]);
    }

    #[test]
    fn test_select_case_match_else() {
        let input = r#"
        SELECT CASE 3
            CASE 1
                PRINT "one"
            CASE ELSE
                PRINT "something else"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["something else"]);
    }

    #[test]
    fn test_select_no_case_only_else() {
        let input = r#"
        SELECT CASE 3
            CASE ELSE
                PRINT "always blue"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["always blue"]);
    }

    #[test]
    fn test_select_is_match() {
        let input = r#"
        SELECT CASE 4
            CASE IS >= 2
                PRINT "greater than 2"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["greater than 2"]);
    }

    #[test]
    fn test_select_is_no_match() {
        let input = r#"
        SELECT CASE 4
            CASE IS >= 5
                PRINT "greater than 5"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, Vec::<String>::new());
    }

    #[test]
    fn test_select_range_within_range() {
        let input = r#"
        SELECT CASE 4
            CASE 2 TO 4
                PRINT "between 2 and 4"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["between 2 and 4"]);
    }

    #[test]
    fn test_select_range_above_range() {
        let input = r#"
        SELECT CASE 4
            CASE 2 TO 3
                PRINT "between 2 and 3"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, Vec::<String>::new());
    }

    #[test]
    fn test_select_range_below_range() {
        let input = r#"
        SELECT CASE 1
            CASE 2 TO 3
                PRINT "between 2 and 3"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, Vec::<String>::new());
    }

    #[test]
    fn test_select_strings() {
        let input = r#"
        SELECT CASE "book"
            CASE "abc" TO "def"
                PRINT "one"
            CASE ELSE
                PRINT "oops"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["one"]);
    }

    #[test]
    fn test_select_double_range_of_integers() {
        let input = r#"
        SELECT CASE 3.14#
            CASE 3 TO 4
                PRINT "pi"
            CASE ELSE
                PRINT "oops"
        END SELECT
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["pi"]);
    }

    #[test]
    fn test_select_wrong_type_in_simple_case() {
        let input = r#"
        SELECT CASE 42
            CASE "book"
                PRINT "hi"
        END SELECT
        "#;
        assert_linter_err!(input, LinterError::TypeMismatch, 3, 18);
    }

    #[test]
    fn test_select_wrong_type_in_range_case_upper() {
        let input = r#"
        SELECT CASE 42
            CASE 1 TO "book"
                PRINT "hi"
        END SELECT
        "#;
        assert_linter_err!(input, LinterError::TypeMismatch, 3, 23);
    }

    #[test]
    fn test_select_wrong_type_in_range_case_lower() {
        let input = r#"
        SELECT CASE 42
            CASE "abc" TO 12
                PRINT "hi"
        END SELECT
        "#;
        assert_linter_err!(input, LinterError::TypeMismatch, 3, 18);
    }

    #[test]
    fn test_select_wrong_type_in_range_case_both() {
        let input = r#"
        SELECT CASE 42
            CASE "abc" TO "def"
                PRINT "hi"
        END SELECT
        "#;
        assert_linter_err!(input, LinterError::TypeMismatch, 3, 18);
    }

    #[test]
    fn test_select_wrong_type_in_is() {
        let input = r#"
        SELECT CASE 42
            CASE IS >= "abc"
                PRINT "hi"
        END SELECT
        "#;
        assert_linter_err!(input, LinterError::TypeMismatch, 3, 24);
    }
}
