#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_has_variable;
    use crate::assert_linter_err;
    use crate::common::*;
    use crate::interpreter::InterpreterError;
    use crate::linter::LinterError;
    use crate::variant::Variant;

    #[test]
    fn test_simple_for_loop_untyped() {
        let input = "
        FOR I = 1 TO 5
            PRINT I
        NEXT
        ";
        let interpreter = interpret(input);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["1", "2", "3", "4", "5"]);
    }

    #[test]
    fn test_simple_for_loop_typed() {
        let input = "
        FOR i% = 1 TO 5
            PRINT i%
        NEXT
        ";
        let interpreter = interpret(input);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["1", "2", "3", "4", "5"]);
    }

    #[test]
    fn test_simple_for_loop_lowercase() {
        let input = "
        FOR i% = 1 TO 5
            PRINT I%
        NEXT
        ";
        let interpreter = interpret(input);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["1", "2", "3", "4", "5"]);
    }

    #[test]
    fn test_simple_for_loop_value_of_variable_after_loop() {
        let input = "
        FOR i% = 1 TO 5
            PRINT i%
        NEXT
        ";
        let interpreter = interpret(input);
        assert_has_variable!(interpreter, "i%", 6);
    }

    #[test]
    fn test_simple_for_loop_value_of_variable_after_loop_never_entering() {
        let input = "
        FOR i% = 1 TO -1
            PRINT i%
        NEXT
        ";
        let interpreter = interpret(input);
        assert_has_variable!(interpreter, "i%", 1);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, Vec::<String>::new());
    }

    #[test]
    fn test_for_loop_with_positive_step() {
        let input = "
        FOR i% = 1 TO 7 STEP 2
            PRINT i%
        NEXT
        ";
        let interpreter = interpret(input);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["1", "3", "5", "7"]);
    }

    #[test]
    fn test_for_loop_with_negative_step() {
        let input = "
        FOR i% = 7 TO -6 STEP -3
            PRINT i%
        NEXT
        ";
        let interpreter = interpret(input);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["7", "4", "1", "-2", "-5"]);
    }

    #[test]
    fn test_for_loop_with_zero_step() {
        let input = "
        FOR i% = 7 TO -6 STEP 0
            PRINT i%
        NEXT
        ";
        assert_eq!(
            interpret_err(input),
            InterpreterError::new_with_pos("Step cannot be zero", Location::new(2, 31))
        );
    }

    #[test]
    fn test_for_loop_with_negative_step_minus_one() {
        let input = "
        FOR i% = 3 TO -3 STEP -1
            PRINT i%
        NEXT
        ";
        let interpreter = interpret(input);
        assert_has_variable!(interpreter, "i%", -4);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["3", "2", "1", "0", "-1", "-2", "-3"]);
    }

    #[test]
    fn test_for_loop_with_specified_next_counter() {
        let input = "
        FOR i% = 1 TO 5
            PRINT i%
        NEXT i%
        ";
        let interpreter = interpret(input);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["1", "2", "3", "4", "5"]);
    }

    #[test]
    fn test_for_loop_with_specified_next_counter_lower_case() {
        let input = "
        FOR i% = 1 TO 5
            PRINT i%
        NEXT I%
        ";
        let interpreter = interpret(input);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["1", "2", "3", "4", "5"]);
    }

    #[test]
    fn test_for_loop_with_wrong_next_counter() {
        let input = "
        FOR i% = 1 TO 5
            PRINT i%
        NEXT i
        ";
        assert_linter_err!(input, LinterError::NextWithoutFor, 4, 14);
    }

    #[test]
    fn test_for_loop_end_expression_evaluated_only_once() {
        let input = "
        N% = 3
        FOR I% = 1 TO N%
            PRINT I%
            N% = N% - 1
        NEXT
        ";
        let interpreter = interpret(input);
        assert_has_variable!(interpreter, "I%", 4);
        assert_has_variable!(interpreter, "N%", 0);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_nested_for_loop() {
        let input = "
        FOR I = 1 to 2
        FOR J = 3 to 4
        PRINT I, J
        NEXT
        NEXT
        ";
        let interpreter = interpret(input);
        assert_eq!(interpreter.stdlib.output, vec!["1 3", "1 4", "2 3", "2 4"]);
    }
}
