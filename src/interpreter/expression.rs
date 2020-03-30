use super::function_context::FunctionImplementation;
use super::*;
use crate::common::Result;
use crate::parser::*;
use std::io::BufRead;

impl<T: BufRead, S: Stdlib> Interpreter<T, S> {
    pub fn evaluate_expression(&mut self, e: &Expression) -> Result<Variant> {
        match e {
            Expression::SingleLiteral(n) => Ok(Variant::from(*n)),
            Expression::DoubleLiteral(n) => Ok(Variant::from(*n)),
            Expression::StringLiteral(s) => Ok(Variant::from(s)),
            Expression::IntegerLiteral(i) => Ok(Variant::from(*i)),
            Expression::LongLiteral(i) => Ok(Variant::from(*i)),
            Expression::VariableName(qn) => self.get_variable(qn),
            Expression::FunctionCall(name, args) => self._evaluate_function_call(name, args),
            Expression::BinaryExpression(op, left, right) => {
                self._evaluate_binary_expression(op, left, right)
            }
        }
    }

    fn _evaluate_function_call(&mut self, name: &QName, args: &Vec<Expression>) -> Result<Variant> {
        match self
            .function_context
            .get_function_implementation(&name.name())
        {
            Some(function_implementation) => {
                self._do_evaluate_function_call(name, function_implementation, args)
            }
            None => self.err(format!("Function {} not defined", name)),
        }
    }

    fn _do_evaluate_function_call(
        &mut self,
        name: &QName,
        function_implementation: FunctionImplementation,
        args: &Vec<Expression>,
    ) -> Result<Variant> {
        let function_parameters: &Vec<QName> = &function_implementation.parameters;
        if function_parameters.len() != args.len() {
            self.err(format!(
                "Function {} expected {} parameters but {} were given",
                name,
                function_parameters.len(),
                args.len()
            ))
        } else {
            self.push_context()?;
            self._populate_new_context(function_parameters, args)?;
            self.statements(&function_implementation.block)?;
            let result = self.get_variable(name);
            self.pop_context()?;
            result
        }
    }

    fn _populate_new_context(
        &mut self,
        parameter_names: &Vec<QName>,
        arguments: &Vec<Expression>,
    ) -> Result<()> {
        let mut i = 0;
        while i < parameter_names.len() {
            let variable_name = &parameter_names[i];
            let variable_value = self.evaluate_expression(&arguments[i])?;
            self.set_variable(variable_name, variable_value)?;
            i += 1;
        }
        Ok(())
    }

    fn _evaluate_binary_expression(
        &mut self,
        op: &Operand,
        left: &Box<Expression>,
        right: &Box<Expression>,
    ) -> Result<Variant> {
        let left_var: Variant = self.evaluate_expression(left)?;
        let right_var: Variant = self.evaluate_expression(right)?;
        match op {
            Operand::LessOrEqualThan => {
                let cmp = left_var.cmp(&right_var)?;
                match cmp {
                    std::cmp::Ordering::Less | std::cmp::Ordering::Equal => Ok(V_TRUE),
                    std::cmp::Ordering::Greater => Ok(V_FALSE),
                }
            }
            Operand::LessThan => {
                let cmp = left_var.cmp(&right_var)?;
                match cmp {
                    std::cmp::Ordering::Less => Ok(V_TRUE),
                    _ => Ok(V_FALSE),
                }
            }
            Operand::Plus => left_var.plus(&right_var),
            Operand::Minus => left_var.minus(&right_var),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::test_utils::*;

    #[test]
    fn test_literals() {
        let stdlib = MockStdlib::new();
        let mut interpreter = Interpreter::new_from_bytes("", stdlib);
        assert_eq!(
            interpreter
                .evaluate_expression(&Expression::from(3.14_f32))
                .unwrap(),
            Variant::from(3.14_f32)
        );
        assert_eq!(
            interpreter
                .evaluate_expression(&Expression::from(3.14))
                .unwrap(),
            Variant::from(3.14)
        );
        assert_eq!(
            interpreter
                .evaluate_expression(&Expression::from("hello"))
                .unwrap(),
            Variant::from("hello")
        );
        assert_eq!(
            interpreter
                .evaluate_expression(&Expression::from(42))
                .unwrap(),
            Variant::from(42)
        );
        assert_eq!(
            interpreter
                .evaluate_expression(&Expression::from(42_i64))
                .unwrap(),
            Variant::from(42_i64)
        );
    }

    mod plus {
        use super::*;

        fn test<TLeft, TRight, TResult>(left: TLeft, right: TRight, expected: TResult)
        where
            Expression: From<TLeft>,
            Expression: From<TRight>,
            Variant: From<TResult>,
        {
            let stdlib = MockStdlib::new();
            let mut interpreter = Interpreter::new_from_bytes("", stdlib);
            assert_eq!(
                interpreter
                    .evaluate_expression(&Expression::plus(
                        Expression::from(left),
                        Expression::from(right)
                    ))
                    .unwrap(),
                Variant::from(expected)
            );
        }

        fn test_err<TLeft, TRight>(left: TLeft, right: TRight)
        where
            Expression: From<TLeft>,
            Expression: From<TRight>,
        {
            let stdlib = MockStdlib::new();
            let mut interpreter = Interpreter::new_from_bytes("", stdlib);
            assert_eq!(
                interpreter
                    .evaluate_expression(&Expression::plus(
                        Expression::from(left),
                        Expression::from(right)
                    ))
                    .unwrap_err(),
                "Type mismatch"
            );
        }

        #[test]
        fn test_left_float() {
            test(1.0_f32, 2.0_f32, 3.0_f32);
            test(1.0_f32, 2.0, 3.0);
            test_err(1.0_f32, "hello");
            test(1.0_f32, 2, 3.0_f32);
            test(1.0_f32, 2_i64, 3.0_f32);
        }

        #[test]
        fn test_left_double() {
            test(1.0, 2.0_f32, 3.0);
            test(1.0, 2.0, 3.0);
            test_err(1.0, "hello");
            test(1.0, 2, 3.0);
            test(1.0, 2_i64, 3.0);
        }

        #[test]
        fn test_left_string() {
            test_err("hello", 3.14_f32);
            test_err("hello", 3.14);
            test("hello", " world", "hello world");
            test_err("hello", 42);
            test_err("hello", 42_i64);
        }

        #[test]
        fn test_left_integer() {
            test(1, 2.0_f32, 3.0_f32);
            test(1, 2.0, 3.0);
            test_err(42, "hello");
            test(1, 2, 3);
            test(1, 2_i64, 3_i64);
        }

        #[test]
        fn test_left_long() {
            test(1_i64, 2.0_f32, 3.0_f32);
            test(1_i64, 2.0, 3.0);
            test_err(1_i64, "hello");
            test(1_i64, 2, 3_i64);
            test(1_i64, 2_i64, 3_i64);
        }
    }

    mod minus {
        use super::*;

        fn test<TLeft, TRight, TResult>(left: TLeft, right: TRight, expected: TResult)
        where
            Expression: From<TLeft>,
            Expression: From<TRight>,
            Variant: From<TResult>,
        {
            let stdlib = MockStdlib::new();
            let mut interpreter = Interpreter::new_from_bytes("", stdlib);
            assert_eq!(
                interpreter
                    .evaluate_expression(&Expression::minus(
                        Expression::from(left),
                        Expression::from(right)
                    ))
                    .unwrap(),
                Variant::from(expected)
            );
        }

        fn test_err<TLeft, TRight>(left: TLeft, right: TRight)
        where
            Expression: From<TLeft>,
            Expression: From<TRight>,
        {
            let stdlib = MockStdlib::new();
            let mut interpreter = Interpreter::new_from_bytes("", stdlib);
            assert_eq!(
                interpreter
                    .evaluate_expression(&Expression::minus(
                        Expression::from(left),
                        Expression::from(right)
                    ))
                    .unwrap_err(),
                "Type mismatch"
            );
        }

        #[test]
        fn test_left_float() {
            test(9.0_f32, 2.0_f32, 7.0_f32);
            test(9.0_f32, 2.0, 7.0);
            test_err(9.0_f32, "hello");
            test(9.0_f32, 2, 7.0_f32);
            test(9.0_f32, 2_i64, 7.0_f32);
        }

        #[test]
        fn test_left_double() {
            test(9.0, 2.0_f32, 7.0);
            test(9.0, 2.0, 7.0);
            test_err(9.0, "hello");
            test(9.0, 2, 7.0);
            test(9.0, 2_i64, 7.0);
        }

        #[test]
        fn test_left_string() {
            test_err("hello", 3.14_f32);
            test_err("hello", 3.14);
            test_err("hello", " world");
            test_err("hello", 42);
            test_err("hello", 42_i64);
        }

        #[test]
        fn test_left_integer() {
            test(9, 2.0_f32, 7.0_f32);
            test(9, 2.0, 7.0);
            test_err(42, "hello");
            test(9, 2, 7);
            test(9, 2_i64, 7_i64);
        }

        #[test]
        fn test_left_long() {
            test(9_i64, 2.0_f32, 7.0_f32);
            test(9_i64, 2.0, 7.0);
            test_err(9_i64, "hello");
            test(9_i64, 2, 7_i64);
            test(9_i64, 2_i64, 7_i64);
        }
    }

    mod less {
        use super::*;

        fn test<TLeft, TRight, TResult>(left: TLeft, right: TRight, expected: TResult)
        where
            Expression: From<TLeft>,
            Expression: From<TRight>,
            Variant: From<TResult>,
        {
            let stdlib = MockStdlib::new();
            let mut interpreter = Interpreter::new_from_bytes("", stdlib);
            assert_eq!(
                interpreter
                    .evaluate_expression(&Expression::less(
                        Expression::from(left),
                        Expression::from(right)
                    ))
                    .unwrap(),
                Variant::from(expected)
            );
        }

        fn test_err<TLeft, TRight>(left: TLeft, right: TRight)
        where
            Expression: From<TLeft>,
            Expression: From<TRight>,
        {
            let stdlib = MockStdlib::new();
            let mut interpreter = Interpreter::new_from_bytes("", stdlib);
            assert_eq!(
                interpreter
                    .evaluate_expression(&Expression::less(
                        Expression::from(left),
                        Expression::from(right)
                    ))
                    .unwrap_err(),
                "Type mismatch"
            );
        }

        #[test]
        fn test_left_float() {
            test(9.0_f32, 2.0_f32, V_FALSE);
            test(9.0_f32, 9.0_f32, V_FALSE);
            test(9.0_f32, 19.0_f32, V_TRUE);

            test(9.0_f32, 2.0, V_FALSE);
            test(9.0_f32, 9.0, V_FALSE);
            test(9.0_f32, 19.0, V_TRUE);

            test_err(9.0_f32, "hello");

            test(9.0_f32, 2, V_FALSE);
            test(9.0_f32, 9, V_FALSE);
            test(9.0_f32, 19, V_TRUE);

            test(9.0_f32, 2_i64, V_FALSE);
            test(9.0_f32, 9_i64, V_FALSE);
            test(9.0_f32, 19_i64, V_TRUE);
        }

        #[test]
        fn test_left_double() {
            test(9.0, 2.0_f32, V_FALSE);
            test(9.0, 9.0_f32, V_FALSE);
            test(9.0, 19.0_f32, V_TRUE);

            test(9.0, 2.0, V_FALSE);
            test(9.0, 9.0, V_FALSE);
            test(9.0, 19.0, V_TRUE);

            test_err(9.0, "hello");

            test(9.0, 2, V_FALSE);
            test(9.0, 9, V_FALSE);
            test(9.0, 19, V_TRUE);

            test(9.0, 2_i64, V_FALSE);
            test(9.0, 9_i64, V_FALSE);
            test(9.0, 19_i64, V_TRUE);
        }

        #[test]
        fn test_left_string() {
            test_err("hello", 3.14_f32);
            test_err("hello", 3.14);
            test("def", "abc", V_FALSE);
            test("def", "def", V_FALSE);
            test("def", "xyz", V_TRUE);
            test_err("hello", 42);
            test_err("hello", 42_i64);
        }

        #[test]
        fn test_left_integer() {
            test(9, 2.0_f32, V_FALSE);
            test(9, 8.9_f32, V_FALSE);
            test(9, 9.0_f32, V_FALSE);
            test(9, 9.1_f32, V_TRUE);
            test(9, 19.0_f32, V_TRUE);

            test(9, 2.0, V_FALSE);
            test(9, 9.0, V_FALSE);
            test(9, 19.0, V_TRUE);

            test_err(9, "hello");

            test(9, 2, V_FALSE);
            test(9, 9, V_FALSE);
            test(9, 19, V_TRUE);

            test(9, 2_i64, V_FALSE);
            test(9, 9_i64, V_FALSE);
            test(9, 19_i64, V_TRUE);
        }

        #[test]
        fn test_left_long() {
            test(9_i64, 2.0_f32, V_FALSE);
            test(9_i64, 8.9_f32, V_FALSE);
            test(9_i64, 9.0_f32, V_FALSE);
            test(9_i64, 9.1_f32, V_TRUE);
            test(9_i64, 19.0_f32, V_TRUE);

            test(9_i64, 2.0, V_FALSE);
            test(9_i64, 9.0, V_FALSE);
            test(9_i64, 19.0, V_TRUE);

            test_err(9_i64, "hello");

            test(9_i64, 2, V_FALSE);
            test(9_i64, 9, V_FALSE);
            test(9_i64, 19, V_TRUE);

            test(9_i64, 2_i64, V_FALSE);
            test(9_i64, 9_i64, V_FALSE);
            test(9_i64, 19_i64, V_TRUE);
        }
    }

    mod lte {
        use super::*;

        fn test<TLeft, TRight, TResult>(left: TLeft, right: TRight, expected: TResult)
        where
            Expression: From<TLeft>,
            Expression: From<TRight>,
            Variant: From<TResult>,
        {
            let stdlib = MockStdlib::new();
            let mut interpreter = Interpreter::new_from_bytes("", stdlib);
            assert_eq!(
                interpreter
                    .evaluate_expression(&Expression::lte(
                        Expression::from(left),
                        Expression::from(right)
                    ))
                    .unwrap(),
                Variant::from(expected)
            );
        }

        fn test_err<TLeft, TRight>(left: TLeft, right: TRight)
        where
            Expression: From<TLeft>,
            Expression: From<TRight>,
        {
            let stdlib = MockStdlib::new();
            let mut interpreter = Interpreter::new_from_bytes("", stdlib);
            assert_eq!(
                interpreter
                    .evaluate_expression(&Expression::lte(
                        Expression::from(left),
                        Expression::from(right)
                    ))
                    .unwrap_err(),
                "Type mismatch"
            );
        }

        #[test]
        fn test_left_float() {
            test(9.0_f32, 2.0_f32, V_FALSE);
            test(9.0_f32, 9.0_f32, V_TRUE);
            test(9.0_f32, 19.0_f32, V_TRUE);

            test(9.0_f32, 2.0, V_FALSE);
            test(9.0_f32, 9.0, V_TRUE);
            test(9.0_f32, 19.0, V_TRUE);

            test_err(9.0_f32, "hello");

            test(9.0_f32, 2, V_FALSE);
            test(9.0_f32, 9, V_TRUE);
            test(9.0_f32, 19, V_TRUE);

            test(9.0_f32, 2_i64, V_FALSE);
            test(9.0_f32, 9_i64, V_TRUE);
            test(9.0_f32, 19_i64, V_TRUE);
        }

        #[test]
        fn test_left_double() {
            test(9.0, 2.0_f32, V_FALSE);
            test(9.0, 9.0_f32, V_TRUE);
            test(9.0, 19.0_f32, V_TRUE);

            test(9.0, 2.0, V_FALSE);
            test(9.0, 9.0, V_TRUE);
            test(9.0, 19.0, V_TRUE);

            test_err(9.0, "hello");

            test(9.0, 2, V_FALSE);
            test(9.0, 9, V_TRUE);
            test(9.0, 19, V_TRUE);

            test(9.0, 2_i64, V_FALSE);
            test(9.0, 9_i64, V_TRUE);
            test(9.0, 19_i64, V_TRUE);
        }

        #[test]
        fn test_left_string() {
            test_err("hello", 3.14_f32);
            test_err("hello", 3.14);
            test("def", "abc", V_FALSE);
            test("def", "def", V_TRUE);
            test("def", "xyz", V_TRUE);
            test_err("hello", 42);
            test_err("hello", 42_i64);
        }

        #[test]
        fn test_left_integer() {
            test(9, 2.0_f32, V_FALSE);
            test(9, 8.9_f32, V_FALSE);
            test(9, 9.0_f32, V_TRUE);
            test(9, 9.1_f32, V_TRUE);
            test(9, 19.0_f32, V_TRUE);

            test(9, 2.0, V_FALSE);
            test(9, 9.0, V_TRUE);
            test(9, 19.0, V_TRUE);

            test_err(9, "hello");

            test(9, 2, V_FALSE);
            test(9, 9, V_TRUE);
            test(9, 19, V_TRUE);

            test(9, 2_i64, V_FALSE);
            test(9, 9_i64, V_TRUE);
            test(9, 19_i64, V_TRUE);
        }

        #[test]
        fn test_left_long() {
            test(9_i64, 2.0_f32, V_FALSE);
            test(9_i64, 8.9_f32, V_FALSE);
            test(9_i64, 9.0_f32, V_TRUE);
            test(9_i64, 9.1_f32, V_TRUE);
            test(9_i64, 19.0_f32, V_TRUE);

            test(9_i64, 2.0, V_FALSE);
            test(9_i64, 9.0, V_TRUE);
            test(9_i64, 19.0, V_TRUE);

            test_err(9_i64, "hello");

            test(9_i64, 2, V_FALSE);
            test(9_i64, 9, V_TRUE);
            test(9_i64, 19, V_TRUE);

            test(9_i64, 2_i64, V_FALSE);
            test(9_i64, 9_i64, V_TRUE);
            test(9_i64, 19_i64, V_TRUE);
        }
    }
}
