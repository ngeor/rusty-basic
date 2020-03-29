use super::context::ReadWriteContext;
use super::*;
use crate::common::Result;
use crate::parser::*;
use std::convert::TryInto;
use std::io::BufRead;
use super::casting::cast;

impl<T: BufRead, S: Stdlib> Interpreter<T, S> {
    pub fn statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::SubCall(name, args) => self.sub_call(name, args),
            Statement::ForLoop(i, a, b, statements) => self.for_loop(i, a, b, statements),
            Statement::IfBlock(i) => self._if_block(i),
            Statement::Assignment(left_side, right_side) => self._assignment(left_side, right_side),
        }
    }

    pub fn statements(&mut self, statements: &Block) -> Result<()> {
        for statement in statements {
            match self.statement(statement) {
                Err(e) => return Err(e),
                Ok(_) => (),
            }
        }
        Ok(())
    }

    fn _if_block(&mut self, if_block: &IfBlock) -> Result<()> {
        let if_condition_expr = &if_block.if_block.condition;
        let if_condition_var = self.evaluate_expression(if_condition_expr)?;
        if if_condition_var.try_into()? {
            self.statements(&if_block.if_block.block)
        } else {
            for else_if_block in &if_block.else_if_blocks {
                let if_condition_expr = &else_if_block.condition;
                let if_condition_var = self.evaluate_expression(if_condition_expr)?;
                if if_condition_var.try_into()? {
                    return self.statements(&else_if_block.block);
                }
            }

            match &if_block.else_block {
                Some(e) => self.statements(&e),
                None => Ok(()),
            }
        }
    }

    fn _assignment(&mut self, left_side: &QName, right_side: &Expression) -> Result<()> {
        let val = self.evaluate_expression(right_side)?;
        let target_type = self.effective_type_qualifier(left_side);
        self.set_variable(left_side, cast(val, target_type)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::test_utils::*;
    use std::str::FromStr;

    mod assignment {
        use super::*;

        fn test_literal(variable_name: QName, raw_literal: &str) -> Result<Variant> {
            let stdlib = MockStdlib::new();
            let input = format!("{} = {}", variable_name, raw_literal);
            let mut interpreter = Interpreter::new_from_bytes(input, stdlib);
            interpreter.interpret()?;
            interpreter.get_variable(&variable_name)
        }

        #[test]
        fn test_assign_literal_to_unqualified_float() {
            assert_eq!(
                test_literal(QName::from_str("X").unwrap(), "1.0").unwrap(),
                Variant::from(1.0_f32)
            );
            assert_eq!(
                test_literal(QName::from_str("X").unwrap(), "1").unwrap(),
                Variant::from(1.0_f32)
            );
            assert_eq!(
                test_literal(QName::from_str("X").unwrap(), "\"hello\"").unwrap_err(),
                "Type mismatch"
            );
        }

        #[test]
        fn test_assign_literal_to_qualified_float() {
            assert_eq!(
                test_literal(QName::from_str("X!").unwrap(), "1.0").unwrap(),
                Variant::from(1.0_f32)
            );
            assert_eq!(
                test_literal(QName::from_str("X!").unwrap(), "1").unwrap(),
                Variant::from(1.0_f32)
            );
            assert_eq!(
                test_literal(QName::from_str("X!").unwrap(), "\"hello\"").unwrap_err(),
                "Type mismatch"
            );
        }

        #[test]
        fn test_assign_literal_to_qualified_double() {
            assert_eq!(
                test_literal(QName::from_str("X#").unwrap(), "1.0").unwrap(),
                Variant::from(1.0)
            );
            assert_eq!(
                test_literal(QName::from_str("X#").unwrap(), "1").unwrap(),
                Variant::from(1.0)
            );
            assert_eq!(
                test_literal(QName::from_str("X#").unwrap(), "\"hello\"").unwrap_err(),
                "Type mismatch"
            );
        }

        #[test]
        fn test_assign_literal_to_qualified_string() {
            assert_eq!(
                test_literal(QName::from_str("A$").unwrap(), "1.0").unwrap_err(),
                "Type mismatch"
            );
            assert_eq!(
                test_literal(QName::from_str("A$").unwrap(), "1").unwrap_err(),
                "Type mismatch"
            );
            assert_eq!(
                test_literal(QName::from_str("A$").unwrap(), "\"hello\"").unwrap(),
                Variant::from("hello")
            );
        }

        #[test]
        fn test_assign_literal_to_qualified_integer() {
            assert_eq!(
                test_literal(QName::from_str("X%").unwrap(), "1.0").unwrap(),
                Variant::from(1)
            );
            assert_eq!(
                test_literal(QName::from_str("X%").unwrap(), "1.1").unwrap(),
                Variant::from(1)
            );
            assert_eq!(
                test_literal(QName::from_str("X%").unwrap(), "1.5").unwrap(),
                Variant::from(2)
            );
            assert_eq!(
                test_literal(QName::from_str("X%").unwrap(), "1.9").unwrap(),
                Variant::from(2)
            );
            assert_eq!(
                test_literal(QName::from_str("X%").unwrap(), "1").unwrap(),
                Variant::from(1)
            );
            assert_eq!(
                test_literal(QName::from_str("X%").unwrap(), "\"hello\"").unwrap_err(),
                "Type mismatch"
            );
        }

        #[test]
        fn test_assign_literal_to_qualified_long() {
            assert_eq!(
                test_literal(QName::from_str("X&").unwrap(), "1.0").unwrap(),
                Variant::from(1_i64)
            );
            assert_eq!(
                test_literal(QName::from_str("X&").unwrap(), "1").unwrap(),
                Variant::from(1_i64)
            );
            assert_eq!(
                test_literal(QName::from_str("X&").unwrap(), "\"hello\"").unwrap_err(),
                "Type mismatch"
            );
        }

        #[test]
        fn test_assign_same_variable_name_different_qualifiers() {
            let stdlib = MockStdlib::new();
            let input = "A = 0.1
A# = 3.14
A$ = \"Hello\"
A% = 1
A& = 100";
            let mut interpreter = Interpreter::new_from_bytes(input, stdlib);
            interpreter.interpret().unwrap();
            assert_eq!(
                interpreter
                    .get_variable(&QName::from_str("A").unwrap())
                    .unwrap(),
                Variant::from(0.1_f32)
            );
            assert_eq!(
                interpreter
                    .get_variable(&QName::from_str("A!").unwrap())
                    .unwrap(),
                Variant::from(0.1_f32)
            );

            let d = match interpreter
                .get_variable(&QName::from_str("A#").unwrap())
                .unwrap()
            {
                Variant::VDouble(d) => d,
                _ => 0.0,
            };
            assert!((d - 3.14).abs() < 0.00001);
            assert_eq!(
                interpreter
                    .get_variable(&QName::from_str("A$").unwrap())
                    .unwrap(),
                Variant::from("Hello")
            );
            assert_eq!(
                interpreter
                    .get_variable(&QName::from_str("A%").unwrap())
                    .unwrap(),
                Variant::from(1)
            );
            assert_eq!(
                interpreter
                    .get_variable(&QName::from_str("A&").unwrap())
                    .unwrap(),
                Variant::from(100_i64)
            );
        }
    }
}
