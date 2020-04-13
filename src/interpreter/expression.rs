use super::variant::{V_FALSE, V_TRUE};
use super::{Interpreter, InterpreterError, Result, Stdlib, VariableGetter, Variant};
use crate::common::HasLocation;
use crate::parser::{ExpressionNode, Operand, OperandNode, UnaryOperand, UnaryOperandNode};

impl<S: Stdlib> Interpreter<S> {
    pub fn evaluate_expression(&mut self, e: &ExpressionNode) -> Result<Variant> {
        match e {
            ExpressionNode::SingleLiteral(n, _) => Ok(Variant::from(*n)),
            ExpressionNode::DoubleLiteral(n, _) => Ok(Variant::from(*n)),
            ExpressionNode::StringLiteral(s, _) => Ok(Variant::from(s)),
            ExpressionNode::IntegerLiteral(i, _) => Ok(Variant::from(*i)),
            ExpressionNode::LongLiteral(i, _) => Ok(Variant::from(*i)),
            ExpressionNode::VariableName(qn) => self.get_variable(qn).map(|x| x.clone()),
            ExpressionNode::FunctionCall(name, args) => self.evaluate_function_call(name, args),
            ExpressionNode::BinaryExpression(op, left, right) => {
                self._evaluate_binary_expression(op, left, right)
            }
            ExpressionNode::UnaryExpression(op, child) => {
                self._evaluate_unary_expression(op, child)
            }
        }
    }

    fn _evaluate_binary_expression(
        &mut self,
        op: &OperandNode,
        left: &Box<ExpressionNode>,
        right: &Box<ExpressionNode>,
    ) -> Result<Variant> {
        let left_var: Variant = self.evaluate_expression(left)?;
        let right_var: Variant = self.evaluate_expression(right)?;
        match op.element() {
            Operand::LessOrEqualThan => {
                let cmp = left_var
                    .cmp(&right_var)
                    .map_err(|e| InterpreterError::new_with_pos(e, op.location()))?;
                match cmp {
                    std::cmp::Ordering::Less | std::cmp::Ordering::Equal => Ok(V_TRUE),
                    std::cmp::Ordering::Greater => Ok(V_FALSE),
                }
            }
            Operand::LessThan => {
                let cmp = left_var
                    .cmp(&right_var)
                    .map_err(|e| InterpreterError::new_with_pos(e, op.location()))?;
                match cmp {
                    std::cmp::Ordering::Less => Ok(V_TRUE),
                    _ => Ok(V_FALSE),
                }
            }
            Operand::Plus => left_var
                .plus(&right_var)
                .map_err(|e| InterpreterError::new_with_pos(e, op.location())),
            Operand::Minus => left_var
                .minus(&right_var)
                .map_err(|e| InterpreterError::new_with_pos(e, op.location())),
        }
    }

    fn _evaluate_unary_expression(
        &mut self,
        op: &UnaryOperandNode,
        child: &Box<ExpressionNode>,
    ) -> Result<Variant> {
        let child_var: Variant = self.evaluate_expression(child)?;
        match op.element() {
            // UnaryOperand::Plus => Ok(child_var),
            UnaryOperand::Minus => Ok(child_var.negate()),
            // UnaryOperand::Not => Ok(if bool::try_from(child_var)? {
            //     V_FALSE
            // } else {
            //     V_TRUE
            // }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_has_variable;
    use crate::common::Location;
    use crate::interpreter::test_utils::*;

    #[test]
    fn test_literals() {
        assert_has_variable!(interpret("X = 3.14"), "X", 3.14_f32);
        assert_has_variable!(interpret("X# = 3.14"), "X#", 3.14);
        assert_has_variable!(interpret("X$ = \"hello\""), "X$", "hello");
        assert_has_variable!(interpret("X% = 42"), "X%", 42);
        assert_has_variable!(interpret("X& = 42"), "X&", 42_i64);
    }

    mod plus {
        use super::*;

        #[test]
        fn test_left_float() {
            assert_has_variable!(interpret("X = 1.1 + 2.1"), "X", 3.2_f32);
            assert_has_variable!(interpret("X = 1.1 + 2.1#"), "X", 3.2_f32);
            assert_has_variable!(interpret("X = 1.1 + 2"), "X", 3.1_f32);
            assert_eq!(
                interpret_err("X = 1.1 + \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 9))
            );
        }

        #[test]
        fn test_left_double() {
            assert_has_variable!(interpret("X# = 1.1# + 2.1"), "X#", 3.2_f64);
            assert_has_variable!(interpret("X# = 1.1 + 2.1#"), "X#", 3.2_f64);
            assert_has_variable!(interpret("X# = 1.1# + 2"), "X#", 3.1_f64);
            assert_eq!(
                interpret_err("X = 1.1# + \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 10))
            );
        }

        #[test]
        fn test_left_string() {
            assert_has_variable!(interpret(r#"X$ = "hello" + " hi""#), "X$", "hello hi");
            assert_eq!(
                interpret_err("X$ = \"hello\" + 1"),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 14))
            );
            assert_eq!(
                interpret_err("X$ = \"hello\" + 1.1"),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 14))
            );
            assert_eq!(
                interpret_err("X$ = \"hello\" + 1.1#"),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 14))
            );
        }

        #[test]
        fn test_left_integer() {
            assert_has_variable!(interpret("X% = 1 + 2.1"), "X%", 3);
            assert_has_variable!(interpret("X% = 1 + 2.5"), "X%", 4);
            assert_has_variable!(interpret("X% = 1 + 2.1#"), "X%", 3);
            assert_has_variable!(interpret("X% = 1 + 2"), "X%", 3);
            assert_eq!(
                interpret_err("X% = 1 + \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 8))
            );
        }

        #[test]
        fn test_left_long() {
            assert_has_variable!(interpret("X& = 1 + 2.1"), "X&", 3_i64);
            assert_has_variable!(interpret("X& = 1 + 2.5"), "X&", 4_i64);
            assert_has_variable!(interpret("X& = 1 + 2.1#"), "X&", 3_i64);
            assert_has_variable!(interpret("X& = 1 + 2"), "X&", 3_i64);
            assert_eq!(
                interpret_err("X& = 1 + \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 8))
            );
        }
    }

    mod minus {
        use super::*;

        #[test]
        fn test_left_float() {
            assert_has_variable!(interpret("X = 5.4 - 2.1"), "X", 3.3_f32);
            assert_has_variable!(interpret("X = 5.4 - 2.1#"), "X", 3.3_f32);
            assert_has_variable!(interpret("X = 5.1 - 2"), "X", 3.1_f32);
            assert_eq!(
                interpret_err("X = 1.1 - \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 9))
            );
        }

        #[test]
        fn test_left_double() {
            assert_has_variable!(interpret("X# = 5.4# - 2.1"), "X#", 3.3_f64);
            assert_has_variable!(interpret("X# = 5.4 - 2.1#"), "X#", 3.3_f64);
            assert_has_variable!(interpret("X# = 5.1# - 2"), "X#", 3.1_f64);
            assert_eq!(
                interpret_err("X = 1.1# - \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 10))
            );
        }

        #[test]
        fn test_left_string() {
            assert_eq!(
                interpret_err("X$ = \"hello\" - \"hi\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 14))
            );
            assert_eq!(
                interpret_err("X$ = \"hello\" - 1"),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 14))
            );
            assert_eq!(
                interpret_err("X$ = \"hello\" - 1.1"),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 14))
            );
            assert_eq!(
                interpret_err("X$ = \"hello\" - 1.1#"),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 14))
            );
        }

        #[test]
        fn test_left_integer() {
            assert_has_variable!(interpret("X% = 5 - 2.1"), "X%", 3);
            assert_has_variable!(interpret("X% = 6 - 2.5"), "X%", 4);
            assert_has_variable!(interpret("X% = 5 - 2.1#"), "X%", 3);
            assert_has_variable!(interpret("X% = 5 - 2"), "X%", 3);
            assert_eq!(
                interpret_err("X% = 1 - \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 8))
            );
        }

        #[test]
        fn test_left_long() {
            assert_has_variable!(interpret("X& = 5 - 2.1"), "X&", 3_i64);
            assert_has_variable!(interpret("X& = 6 - 2.5"), "X&", 4_i64);
            assert_has_variable!(interpret("X& = 5 - 2.1#"), "X&", 3_i64);
            assert_has_variable!(interpret("X& = 5 - 2"), "X&", 3_i64);
            assert_eq!(
                interpret_err("X& = 1 - \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 8))
            );
        }
    }

    macro_rules! assert_condition {
        ($condition:expr) => {
            let program = format!(
                "
            IF {} THEN
            ELSE
                PRINT \"hi\"
            END IF
            ",
                $condition
            );
            if interpret(program).stdlib.output.len() > 0 {
                panic!(format!(
                    "Expected condition to be true but was false: {}",
                    $condition
                ))
            }
        };
    }

    macro_rules! assert_condition_false {
        ($condition:expr) => {
            let program = format!(
                "
            IF {} THEN
                PRINT \"hi\"
            END IF
            ",
                $condition
            );
            if interpret(program).stdlib.output.len() > 0 {
                panic!(format!(
                    "Expected condition to be false but was true: {}",
                    $condition
                ))
            }
        };
    }

    macro_rules! assert_condition_err {
        ($condition:expr) => {
            let program = format!(
                "
            IF {} THEN
                PRINT \"hi\"
            END IF
            ",
                $condition
            );
            let e = interpret_err(program);
            assert_eq!("Type mismatch", e.message());
        };
    }

    mod less {
        use super::*;

        #[test]
        fn test_left_float() {
            assert_condition_false!("9.1 < 2.1");
            assert_condition_false!("9.1 < 9.1");
            assert_condition!("9.1 < 19.1");

            assert_condition_false!("9.1 < 2");
            assert_condition_false!("9.1 < 9");
            assert_condition!("9.1 < 19");

            assert_condition_err!("9.1 < \"hello\"");

            assert_condition_false!("9.1 < 2.1#");
            assert_condition_false!("9.1 < 9.1#");
            assert_condition!("9.1 < 19.1#");
        }

        #[test]
        fn test_left_double() {
            assert_condition_false!("9.1# < 2.1");
            assert_condition_false!("9.1# < 9.1");
            assert_condition!("9.1# < 19.1");

            assert_condition_false!("9.1# < 2");
            assert_condition_false!("9.1# < 9");
            assert_condition!("9.1# < 19");

            assert_condition_err!("9.1# < \"hello\"");

            assert_condition_false!("9.1# < 2.1#");
            assert_condition_false!("9.1# < 9.1#");
            assert_condition!("9.1# < 19.1#");
        }

        #[test]
        fn test_left_string() {
            assert_condition_err!("\"hello\" < 3.14");
            assert_condition_err!("\"hello\" < 3");
            assert_condition_err!("\"hello\" < 3.14#");

            assert_condition_false!("\"def\" < \"abc\"");
            assert_condition_false!("\"def\" < \"def\"");
            assert_condition!("\"def\" < \"xyz\"");
        }

        #[test]
        fn test_left_integer() {
            assert_condition_false!("9 < 2.1");
            assert_condition_false!("9 < 8.9");
            assert_condition_false!("9 < 9.0");
            assert_condition!("9 < 9.1");
            assert_condition!("9 < 19.1");

            assert_condition_false!("9 < 2");
            assert_condition_false!("9 < 9");
            assert_condition!("9 < 19");

            assert_condition_err!("9 < \"hello\"");

            assert_condition_false!("9 < 2.1#");
            assert_condition!("9 < 9.1#");
            assert_condition!("9 < 19.1#");
        }
    }

    mod lte {
        use super::*;

        #[test]
        fn test_left_float() {
            assert_condition_false!("9.1 <= 2.1");
            assert_condition!("9.1 <= 9.1");
            assert_condition!("9.1 <= 19.1");

            assert_condition_false!("9.1 <= 2");
            assert_condition_false!("9.1 <= 9");
            assert_condition!("9.1 <= 19");

            assert_condition_err!("9.1 <= \"hello\"");

            assert_condition_false!("9.1 <= 2.1#");
            assert_condition!("9.1 <= 9.1#");
            assert_condition!("9.1 <= 19.1#");
        }

        #[test]
        fn test_left_double() {
            assert_condition_false!("9.1# <= 2.1");
            assert_condition!("9.1# <= 9.1");
            assert_condition!("9.1# <= 19.1");

            assert_condition_false!("9.1# <= 2");
            assert_condition_false!("9.1# <= 9");
            assert_condition!("9.1# <= 19");

            assert_condition_err!("9.1# <= \"hello\"");

            assert_condition_false!("9.1# <= 2.1#");
            assert_condition!("9.1# <= 9.1#");
            assert_condition!("9.1# <= 19.1#");
        }

        #[test]
        fn test_left_string() {
            assert_condition_err!("\"hello\" <= 3.14");
            assert_condition_err!("\"hello\" <= 3");
            assert_condition_err!("\"hello\" <= 3.14#");

            assert_condition_false!("\"def\" <= \"abc\"");
            assert_condition!("\"def\" <= \"def\"");
            assert_condition!("\"def\" <= \"xyz\"");
        }

        #[test]
        fn test_left_integer() {
            assert_condition_false!("9 <= 2.1");
            assert_condition_false!("9 <= 8.9");
            assert_condition!("9 <= 9.0");
            assert_condition!("9 <= 9.1");
            assert_condition!("9 <= 19.1");

            assert_condition_false!("9 <= 2");
            assert_condition!("9 <= 9");
            assert_condition!("9 <= 19");

            assert_condition_err!("9 <= \"hello\"");

            assert_condition_false!("9 <= 2.1#");
            assert_condition!("9 <= 9.1#");
            assert_condition!("9 <= 19.1#");
        }
    }
}
