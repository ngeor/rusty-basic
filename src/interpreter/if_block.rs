#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

    #[test]
    fn test_if_block_true() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_block_false() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, Vec::<String>::new());
    }

    #[test]
    fn test_if_else_block_true() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSE
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_else_block_false() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSE
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["bye"]);
    }

    #[test]
    fn test_if_elseif_block_true_true() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_elseif_block_true_false() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSEIF 2 < 1 THEN
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_elseif_block_false_true() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["bye"]);
    }

    #[test]
    fn test_if_elseif_block_false_false() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 2 < 1 THEN
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, Vec::<String>::new());
    }

    #[test]
    fn test_if_elseif_else_block_true_true() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        ELSE
            PRINT \"else\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_elseif_else_block_true_false() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSEIF 2 < 1 THEN
            PRINT \"bye\"
        ELSE
            PRINT \"else\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_elseif_else_block_false_true() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        ELSE
            PRINT \"else\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["bye"]);
    }

    #[test]
    fn test_if_elseif_else_block_false_false() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 2 < 1 THEN
            PRINT \"bye\"
        ELSE
            PRINT \"else\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["else"]);
    }

    #[test]
    fn test_if_multiple_elseif_block() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        ELSEIF 1 < 2 THEN
            PRINT \"else if 2\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["bye"]);
    }

    #[test]
    fn test_single_line_if() {
        let input = r#"
        IF 1 THEN PRINT "hello"
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
        let input = r#"
        IF 0 THEN PRINT "hello"
        "#;
        assert_eq!(interpret(input).stdlib.output.len(), 0);
        let input = r#"
        PRINT "before"
        IF 1 THEN PRINT "hello"
        PRINT "after"
        "#;
        assert_eq!(
            interpret(input).stdlib.output,
            vec!["before", "hello", "after"]
        );
        let input = r#"
        PRINT "before"
        IF 0 THEN PRINT "hello"
        PRINT "after"
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["before", "after"]);
    }
}
