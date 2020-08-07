#[cfg(test)]
mod tests {
    use crate::assert_prints;

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
        assert_prints!(input, "one");
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
        assert_prints!(input, "two");
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
        assert_prints!(input; nothing);
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
        assert_prints!(input, "one");
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
        assert_prints!(input, "something else");
    }

    #[test]
    fn test_select_no_case_only_else() {
        let input = r#"
        SELECT CASE 3
            CASE ELSE
                PRINT "always blue"
        END SELECT
        "#;
        assert_prints!(input, "always blue");
    }

    #[test]
    fn test_select_is_match() {
        let input = r#"
        SELECT CASE 4
            CASE IS >= 2
                PRINT "greater than 2"
        END SELECT
        "#;
        assert_prints!(input, "greater than 2");
    }

    #[test]
    fn test_select_is_no_match() {
        let input = r#"
        SELECT CASE 4
            CASE IS >= 5
                PRINT "greater than 5"
        END SELECT
        "#;
        assert_prints!(input; nothing);
    }

    #[test]
    fn test_select_range_within_range() {
        let input = r#"
        SELECT CASE 4
            CASE 2 TO 4
                PRINT "between 2 and 4"
        END SELECT
        "#;
        assert_prints!(input, "between 2 and 4");
    }

    #[test]
    fn test_select_range_above_range() {
        let input = r#"
        SELECT CASE 4
            CASE 2 TO 3
                PRINT "between 2 and 3"
        END SELECT
        "#;
        assert_prints!(input; nothing);
    }

    #[test]
    fn test_select_range_below_range() {
        let input = r#"
        SELECT CASE 1
            CASE 2 TO 3
                PRINT "between 2 and 3"
        END SELECT
        "#;
        assert_prints!(input; nothing);
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
        assert_prints!(input, "one");
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
        assert_prints!(input, "pi");
    }
}
