use crate::common::*;

pub type InterpreterError = ErrorEnvelope<String>;

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
}
