#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::linter::LinterError;

    #[test]
    fn go_to_label_go_to_is_before_label_declaration() {
        let input = r#"
        PRINT "a"
        GOTO Jump
        PRINT "b"
        Jump:
        PRINT "c"
        "#;
        assert_prints!(input, "a", "c");
    }

    #[test]
    fn go_to_label_go_to_is_after_label_declaration() {
        let input = r#"
        X = 0
        Jump:
        PRINT X
        X = X + 1
        IF X <= 1 THEN
            GOTO Jump
        END IF
        "#;
        assert_prints!(input, "0", "1");
    }

    #[test]
    fn go_to_label_go_to_in_single_line_if_then() {
        let input = r#"
        X = 0
        Jump:
        PRINT X
        X = X + 1
        IF X <= 1 THEN GOTO Jump
        "#;
        assert_prints!(input, "0", "1");
    }

    #[test]
    fn go_to_missing_label() {
        let input = "
        GOTO Jump
        ";
        assert_linter_err!(input, LinterError::LabelNotDefined, 2, 9);
    }

    #[test]
    fn go_to_duplicate_label() {
        let input = "
        GOTO Jump
        Jump:
        Jump:
        ";
        assert_linter_err!(input, LinterError::DuplicateLabel, 4, 9);
    }
}
