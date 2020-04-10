use super::{
    Interpreter, InterpreterError, Result, Stdlib, VariableGetter, VariableSetter, Variant,
};
use crate::common::{HasLocation, StripLocation};
use crate::parser::{ForLoopNode, NameNode};
use std::cmp::Ordering;

impl<S: Stdlib> Interpreter<S> {
    pub fn for_loop(&mut self, for_loop: &ForLoopNode) -> Result<()> {
        let start = self.evaluate_expression(&for_loop.lower_bound)?;
        if !start.is_numeric() {
            return Err(InterpreterError::new_with_pos(
                "Start expression was not numeric",
                for_loop.pos,
            ));
        }

        let stop = self.evaluate_expression(&for_loop.upper_bound)?;
        if !stop.is_numeric() {
            return Err(InterpreterError::new_with_pos(
                "Stop expression was not numeric",
                for_loop.pos,
            ));
        }

        let counter_var_name = &for_loop.variable_name;
        let statements = &for_loop.statements;
        let step = match &for_loop.step {
            Some(s) => self.evaluate_expression(s)?,
            None => Variant::from(1),
        };
        if !step.is_numeric() {
            return Err(InterpreterError::new_with_pos(
                "Step expression was not numeric",
                for_loop.pos,
            ));
        }

        self._validate_next_counter(&for_loop)?;

        let step_sign = step
            .cmp(&Variant::from(0))
            .map_err(|e| InterpreterError::new_with_pos(e, for_loop.pos))?;

        match step_sign {
            Ordering::Greater => {
                self.set_variable(counter_var_name, start.clone())?;
                while self._is_less_or_equal(counter_var_name, &stop)? {
                    self.statements(statements)?;
                    self._inc_variable(counter_var_name, &step)?;
                }
                Ok(())
            }
            Ordering::Less => {
                self.set_variable(counter_var_name, start.clone())?;
                while self._is_greater_or_equal(counter_var_name, &stop)? {
                    self.statements(statements)?;
                    self._inc_variable(counter_var_name, &step)?;
                }
                Ok(())
            }
            Ordering::Equal => Err(InterpreterError::new_with_pos(
                "Step cannot be zero",
                for_loop.pos,
            )),
        }
    }

    fn _inc_variable(&mut self, variable_name: &NameNode, step: &Variant) -> Result<()> {
        let existing_value = self.get_variable(variable_name)?;
        let new_value = existing_value
            .plus(step)
            .map_err(|e| InterpreterError::new_with_pos(e, variable_name.location()))?;
        self.set_variable(variable_name, new_value).map(|_| ())
    }

    fn _is_less_or_equal(&self, variable_name: &NameNode, stop: &Variant) -> Result<bool> {
        self.get_variable(variable_name)?
            .cmp(&stop)
            .map(|o| o != std::cmp::Ordering::Greater)
            .map_err(|e| InterpreterError::new_with_pos(e, variable_name.location()))
    }

    fn _is_greater_or_equal(&self, variable_name: &NameNode, stop: &Variant) -> Result<bool> {
        self.get_variable(variable_name)?
            .cmp(&stop)
            .map(|o| o != std::cmp::Ordering::Less)
            .map_err(|e| InterpreterError::new_with_pos(e, variable_name.location()))
    }

    fn _validate_next_counter(&self, for_loop: &ForLoopNode) -> Result<()> {
        if self._are_different_variable_opt(&for_loop.variable_name, &for_loop.next_counter) {
            Err(InterpreterError::new_with_pos(
                "NEXT without FOR",
                for_loop.next_counter.as_ref().unwrap().location(),
            ))
        } else {
            Ok(())
        }
    }

    fn _are_different_variable_opt(&self, left: &NameNode, right: &Option<NameNode>) -> bool {
        match right.as_ref() {
            None => false,
            Some(r) => self._are_different_variable(left, r),
        }
    }

    fn _are_different_variable(&self, left: &NameNode, right: &NameNode) -> bool {
        let left_qualified_name_node = left.resolve(self);
        let right_qualified_name_node = right.resolve(self);
        let left_qualified_name = left_qualified_name_node.strip_location();
        let right_qualified_name = right_qualified_name_node.strip_location();
        left_qualified_name != right_qualified_name
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::InterpreterError;
    use crate::common::Location;

    #[test]
    fn test_simple_for_loop() {
        let input = "
        FOR i% = 1 TO 5
            PRINT i%
        NEXT
        ";
        let interpreter = interpret(input, MockStdlib::new()).unwrap();
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
        let interpreter = interpret(input, MockStdlib::new()).unwrap();
        interpreter.has_variable("i%", 6);
    }

    #[test]
    fn test_simple_for_loop_value_of_variable_after_loop_never_entering() {
        let input = "
        FOR i% = 1 TO -1
            PRINT i%
        NEXT
        ";
        let interpreter = interpret(input, MockStdlib::new()).unwrap();
        interpreter.has_variable("i%", 1);
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
        let interpreter = interpret(input, MockStdlib::new()).unwrap();
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
        let interpreter = interpret(input, MockStdlib::new()).unwrap();
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["7", "4", "1", "-2", "-5"]);
    }

    #[test]
    fn test_for_loop_with_negative_step_minus_one() {
        let input = "
        FOR i% = 3 TO -3 STEP -1
            PRINT i%
        NEXT
        ";
        let interpreter = interpret(input, MockStdlib::new()).unwrap();
        interpreter.has_variable("i%", -4);
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
        let interpreter = interpret(input, MockStdlib::new()).unwrap();
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
        match interpret(input, MockStdlib::new()) {
            Ok(_) => panic!("should have failed"),
            Err(err) => {
                assert_eq!(
                    err,
                    InterpreterError::new_with_pos("NEXT without FOR", Location::new(4, 14))
                );
            }
        }
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
        let interpreter = interpret(input, MockStdlib::new()).unwrap();
        interpreter.has_variable("I%", 4);
        interpreter.has_variable("N%", 0);
        let stdlib = interpreter.stdlib;
        assert_eq!(stdlib.output, vec!["1", "2", "3"]);
    }
}
