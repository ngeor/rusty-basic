use super::context::ReadWriteContext;
use super::*;
use crate::common::Result;
use crate::parser::*;
use std::convert::TryInto;
use std::io::BufRead;

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

fn cast(value: Variant, target_type: TypeQualifier) -> Result<Variant> {
    match value {
        Variant::VSingle(f) => match target_type {
            TypeQualifier::BangSingle => Ok(value),
            TypeQualifier::HashDouble => Ok(Variant::VDouble(f.try_cast()?)),
            TypeQualifier::DollarString => Err("Type mismatch".to_string()),
            TypeQualifier::PercentInteger => Ok(Variant::VInteger(f.try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(Variant::VLong(f.try_cast()?)),
        },
        Variant::VDouble(f) => match target_type {
            TypeQualifier::BangSingle => Ok(Variant::VSingle(f.try_cast()?)),
            TypeQualifier::HashDouble => Ok(value),
            TypeQualifier::DollarString => Err("Type mismatch".to_string()),
            TypeQualifier::PercentInteger => Ok(Variant::VInteger(f.try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(Variant::VLong(f.try_cast()?)),
        },
        Variant::VString(_) => match target_type {
            TypeQualifier::DollarString => Ok(value),
            _ => Err("Type mismatch".to_string()),
        },
        Variant::VInteger(f) => match target_type {
            TypeQualifier::BangSingle => Ok(Variant::VSingle(f.try_cast()?)),
            TypeQualifier::HashDouble => Ok(Variant::VDouble(f.try_cast()?)),
            TypeQualifier::DollarString => Err("Type mismatch".to_string()),
            TypeQualifier::PercentInteger => Ok(value),
            TypeQualifier::AmpersandLong => Ok(Variant::VLong(f.try_cast()?)),
        },
        Variant::VLong(f) => match target_type {
            TypeQualifier::BangSingle => Ok(Variant::VSingle(f.try_cast()?)),
            TypeQualifier::HashDouble => Ok(Variant::VDouble(f.try_cast()?)),
            TypeQualifier::DollarString => Err("Type mismatch".to_string()),
            TypeQualifier::PercentInteger => Ok(Variant::VInteger(f.try_cast()?)),
            TypeQualifier::AmpersandLong => Ok(value),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::test_utils::*;
    use std::str::FromStr;

    mod casts {
        use super::*;

        mod from_float {
            use super::*;

            #[test]
            fn to_float() {
                assert_eq!(
                    cast(Variant::from(1.0_f32), TypeQualifier::BangSingle).unwrap(),
                    Variant::from(1.0_f32)
                );
            }

            #[test]
            fn to_double() {
                assert_eq!(
                    cast(Variant::from(1.0_f32), TypeQualifier::HashDouble).unwrap(),
                    Variant::from(1.0)
                );
            }

            #[test]
            fn to_string() {
                cast(Variant::from(1.0_f32), TypeQualifier::DollarString)
                    .expect_err("Type mismatch");
            }

            #[test]
            fn to_integer() {
                assert_eq!(
                    cast(Variant::from(1.0_f32), TypeQualifier::PercentInteger).unwrap(),
                    Variant::from(1)
                );
            }

            #[test]
            fn to_long() {
                assert_eq!(
                    cast(Variant::from(1.0_f32), TypeQualifier::AmpersandLong).unwrap(),
                    Variant::from(1_i64)
                );
            }
        }

        mod from_double {
            use super::*;

            #[test]
            fn to_float() {
                assert_eq!(
                    cast(Variant::from(1.0), TypeQualifier::BangSingle).unwrap(),
                    Variant::from(1.0_f32)
                );
            }

            #[test]
            fn to_double() {
                assert_eq!(
                    cast(Variant::from(1.0), TypeQualifier::HashDouble).unwrap(),
                    Variant::from(1.0)
                );
            }

            #[test]
            fn to_string() {
                cast(Variant::from(1.0), TypeQualifier::DollarString).expect_err("Type mismatch");
            }

            #[test]
            fn to_integer() {
                assert_eq!(
                    cast(Variant::from(1.0), TypeQualifier::PercentInteger).unwrap(),
                    Variant::from(1)
                );
            }

            #[test]
            fn to_long() {
                assert_eq!(
                    cast(Variant::from(1.0), TypeQualifier::AmpersandLong).unwrap(),
                    Variant::from(1_i64)
                );
            }
        }

        mod from_string {
            use super::*;

            #[test]
            fn to_float() {
                cast(Variant::from("hello"), TypeQualifier::BangSingle).expect_err("Type mismatch");
            }

            #[test]
            fn to_double() {
                cast(Variant::from("hello"), TypeQualifier::HashDouble).expect_err("Type mismatch");
            }

            #[test]
            fn to_string() {
                assert_eq!(
                    cast(Variant::from("hello"), TypeQualifier::DollarString).unwrap(),
                    Variant::from("hello")
                );
            }

            #[test]
            fn to_integer() {
                cast(Variant::from("hello"), TypeQualifier::PercentInteger)
                    .expect_err("Type mismatch");
            }

            #[test]
            fn to_long() {
                cast(Variant::from("hello"), TypeQualifier::AmpersandLong)
                    .expect_err("Type mismatch");
            }
        }

        mod from_integer {
            use super::*;

            #[test]
            fn to_float() {
                assert_eq!(
                    cast(Variant::from(1), TypeQualifier::BangSingle).unwrap(),
                    Variant::from(1.0_f32)
                );
            }

            #[test]
            fn to_double() {
                assert_eq!(
                    cast(Variant::from(1), TypeQualifier::HashDouble).unwrap(),
                    Variant::from(1.0)
                );
            }

            #[test]
            fn to_string() {
                cast(Variant::from(1), TypeQualifier::DollarString).expect_err("Type mismatch");
            }

            #[test]
            fn to_integer() {
                assert_eq!(
                    cast(Variant::from(1), TypeQualifier::PercentInteger).unwrap(),
                    Variant::from(1)
                );
            }

            #[test]
            fn to_long() {
                assert_eq!(
                    cast(Variant::from(1), TypeQualifier::AmpersandLong).unwrap(),
                    Variant::from(1_i64)
                );
            }
        }

        mod from_long {
            use super::*;

            #[test]
            fn to_float() {
                assert_eq!(
                    cast(Variant::from(1_i64), TypeQualifier::BangSingle).unwrap(),
                    Variant::from(1.0_f32)
                );
            }

            #[test]
            fn to_double() {
                assert_eq!(
                    cast(Variant::from(1_i64), TypeQualifier::HashDouble).unwrap(),
                    Variant::from(1.0)
                );
            }

            #[test]
            fn to_string() {
                cast(Variant::from(1_i64), TypeQualifier::DollarString).expect_err("Type mismatch");
            }

            #[test]
            fn to_integer() {
                assert_eq!(
                    cast(Variant::from(1_i64), TypeQualifier::PercentInteger).unwrap(),
                    Variant::from(1)
                );
            }

            #[test]
            fn to_long() {
                assert_eq!(
                    cast(Variant::from(1_i64), TypeQualifier::AmpersandLong).unwrap(),
                    Variant::from(1_i64)
                );
            }
        }
    }

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
