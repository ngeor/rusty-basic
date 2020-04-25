use super::variant::{V_FALSE, V_TRUE};
use super::{Interpreter, InterpreterError, Result, Stdlib, Variant};
use crate::common::HasLocation;
use crate::parser::{ExpressionNode, Operand, OperandNode, UnaryOperand, UnaryOperandNode};

impl<S: Stdlib> Interpreter<S> {
    pub fn evaluate_expression(&mut self, e: &ExpressionNode) -> Result<Variant> {
        self._evaluate_expression(e, false)
    }

    pub fn evaluate_const_expression(&mut self, e: &ExpressionNode) -> Result<Variant> {
        self._evaluate_expression(e, true)
    }

    fn _evaluate_expression(
        &mut self,
        e: &ExpressionNode,
        only_constants: bool,
    ) -> Result<Variant> {
        match e {
            ExpressionNode::SingleLiteral(n, _) => Ok(Variant::from(*n)),
            ExpressionNode::DoubleLiteral(n, _) => Ok(Variant::from(*n)),
            ExpressionNode::StringLiteral(s, _) => Ok(Variant::from(s)),
            ExpressionNode::IntegerLiteral(i, _) => Ok(Variant::from(*i)),
            ExpressionNode::LongLiteral(i, _) => Ok(Variant::from(*i)),
            ExpressionNode::VariableName(n) => {
                if only_constants {
                    self.context.get_const(n).map(|x| x.clone())
                } else {
                    self.context.get_or_default(n)
                }
            }
            ExpressionNode::FunctionCall(n, args) => {
                if only_constants {
                    Err(InterpreterError::new_with_pos(
                        "Invalid constant",
                        e.location(),
                    ))
                } else {
                    self.evaluate_function_call(n, args)
                }
            }
            ExpressionNode::BinaryExpression(op, left, right) => {
                self._evaluate_binary_expression(op, left, right, only_constants)
            }
            ExpressionNode::UnaryExpression(op, child) => {
                self._evaluate_unary_expression(op, child, only_constants)
            }
        }
    }

    fn _evaluate_binary_expression(
        &mut self,
        op: &OperandNode,
        left: &Box<ExpressionNode>,
        right: &Box<ExpressionNode>,
        only_constants: bool,
    ) -> Result<Variant> {
        let left_var: Variant = self._evaluate_expression(left, only_constants)?;
        let right_var: Variant = self._evaluate_expression(right, only_constants)?;
        match op.as_ref() {
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
        only_constants: bool,
    ) -> Result<Variant> {
        let child_var: Variant = self._evaluate_expression(child, only_constants)?;
        match op.as_ref() {
            // UnaryOperand::Plus => Ok(child_var),
            UnaryOperand::Minus => child_var
                .negate()
                .map_err(|e| InterpreterError::new_with_pos(e, op.location())),
            UnaryOperand::Not => child_var
                .unary_not()
                .map_err(|e| InterpreterError::new_with_pos(e, op.location())),
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

    mod binary_plus {
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

    mod binary_minus {
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

    mod unary_minus {
        use super::*;

        #[test]
        fn test_unary_minus_float() {
            assert_has_variable!(interpret("X = -1.1"), "X", -1.1_f32);
            assert_has_variable!(interpret("X = -1.1#"), "X", -1.1_f32);
            assert_has_variable!(interpret("X = -1"), "X", -1.0_f32);
            assert_eq!(
                interpret_err("X = -\"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 5))
            );
        }

        #[test]
        fn test_unary_minus_integer() {
            assert_has_variable!(interpret("X% = -1.1"), "X%", -1);
            assert_has_variable!(interpret("X% = -1.1#"), "X%", -1);
            assert_has_variable!(interpret("X% = -1"), "X%", -1);
            assert_eq!(
                interpret_err("X% = -\"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 6))
            );
        }
    }

    mod unary_not {
        use super::*;

        #[test]
        fn test_unary_not_float() {
            assert_has_variable!(interpret("X = NOT 3.14"), "X", -4.0_f32);
            assert_has_variable!(interpret("X = NOT 3.5#"), "X", -5.0_f32);
            assert_has_variable!(interpret("X = NOT -1.1"), "X", 0.0_f32);
            assert_has_variable!(interpret("X = NOT -1.5"), "X", 1.0_f32);
            assert_eq!(
                interpret_err("X = NOT \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 5))
            );
        }

        #[test]
        fn test_unary_not_integer() {
            assert_has_variable!(interpret("X% = NOT 1"), "X%", -2);
            assert_has_variable!(interpret("X% = NOT 0"), "X%", -1);
            assert_has_variable!(interpret("X% = NOT -1"), "X%", 0);
            assert_has_variable!(interpret("X% = NOT -2"), "X%", 1);
            assert_eq!(
                interpret_err("X% = NOT \"hello\""),
                InterpreterError::new_with_pos("Type mismatch", Location::new(1, 6))
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
