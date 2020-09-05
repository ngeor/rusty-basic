#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

    #[test]
    fn on_error_go_to_label() {
        let input = r#"
        ON ERROR GOTO ErrTrap
        Environ "ShouldHaveAnEqualsSignInHereSomewhere"
        PRINT "Will not print this"
        SYSTEM
        ErrTrap:
            PRINT "Saved by the bell"
        "#;
        let interpreter = interpret(input);
        assert_eq!(interpreter.stdlib.output, vec!["Saved by the bell"]);
    }

    #[test]
    fn on_error_go_to_label_with_dots_in_label_name() {
        let input = r#"
        ON ERROR GOTO Err.Trap
        PRINT 1 / 0
        SYSTEM
        Err.Trap:
            PRINT "Almost divided by zero"
        "#;
        let interpreter = interpret(input);
        assert_eq!(interpreter.stdlib.output, vec!["Almost divided by zero"]);
    }
}
