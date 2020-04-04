use super::*;
use crate::common::Result;
use crate::parser::ForLoop;
use std::cmp::Ordering;

impl<S: Stdlib> Interpreter<S> {
    pub fn for_loop(&mut self, for_loop: &ForLoop) -> Result<()> {
        let start = self.evaluate_expression(&for_loop.lower_bound)?;
        if !start.is_numeric() {
            return self.err("Start expression was not numeric");
        }

        let stop = self.evaluate_expression(&for_loop.upper_bound)?;
        if !stop.is_numeric() {
            return self.err("Stop expression was not numeric");
        }

        let counter_var_name = &for_loop.variable_name;
        let statements = &for_loop.statements;
        let step = match &for_loop.step {
            Some(s) => self.evaluate_expression(s)?,
            None => Variant::from(1),
        };
        if !step.is_numeric() {
            return self.err("Step expression was not numeric");
        }

        match &for_loop.next_counter {
            None => (),
            Some(next_counter_name) => {
                if !self._are_same_variable(next_counter_name, counter_var_name) {
                    return self.err("NEXT without FOR");
                }
            }
        }

        let step_sign = step.cmp(&Variant::from(0))?;

        match step_sign {
            Ordering::Greater => {
                self.set_variable(counter_var_name, start.clone());
                while self._is_less_or_equal(counter_var_name, &stop)? {
                    self.statements(statements)?;
                    self._inc_variable(counter_var_name, &step)?;
                }
                Ok(())
            }
            Ordering::Less => {
                self.set_variable(counter_var_name, start.clone());
                while self._is_greater_or_equal(counter_var_name, &stop)? {
                    self.statements(statements)?;
                    self._inc_variable(counter_var_name, &step)?;
                }
                Ok(())
            }
            Ordering::Equal => self.err("Step cannot be zero"),
        }
    }

    fn _inc_variable(&mut self, variable_name: &QName, step: &Variant) -> Result<()> {
        let existing_value = self.get_variable(variable_name)?;
        let new_value = existing_value.plus(step)?;
        self.set_variable(variable_name, new_value);
        Ok(())
    }

    fn _is_less_or_equal(&self, variable_name: &QName, stop: &Variant) -> Result<bool> {
        self.get_variable(variable_name)?
            .cmp(&stop)
            .map(|o| o != std::cmp::Ordering::Greater)
    }

    fn _is_greater_or_equal(&self, variable_name: &QName, stop: &Variant) -> Result<bool> {
        self.get_variable(variable_name)?
            .cmp(&stop)
            .map(|o| o != std::cmp::Ordering::Less)
    }

    fn _are_same_variable(&self, left: &QName, right: &QName) -> bool {
        match left {
            QName::Untyped(left_bare_name) => match right {
                QName::Untyped(right_bare_name) => left_bare_name == right_bare_name,
                QName::Typed(right_qualified_name) => {
                    left_bare_name == &right_qualified_name.name
                        && self.effective_type_qualifier(left_bare_name)
                            == right_qualified_name.qualifier
                }
            },
            QName::Typed(left_qualified_name) => match right {
                QName::Untyped(right_bare_name) => {
                    &left_qualified_name.name == right_bare_name
                        && left_qualified_name.qualifier
                            == self.effective_type_qualifier(right_bare_name)
                }
                QName::Typed(right_qualified_name) => left_qualified_name == right_qualified_name,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

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
                assert_eq!(err, "NEXT without FOR");
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
