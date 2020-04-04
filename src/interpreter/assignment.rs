use super::casting::cast;
use super::*;
use crate::common::Result;
use crate::parser::*;

impl<S: Stdlib> Interpreter<S> {
    pub fn assignment(&mut self, left_side: &QName, right_side: &Expression) -> Result<()> {
        let val: Variant = self.evaluate_expression(right_side)?;
        match left_side {
            QName::Untyped(bare_name) => self._assign_untyped(bare_name, val),
            QName::Typed(qualified_name) => self._assign_typed(qualified_name, val),
        }
    }

    fn _assign_untyped(&mut self, bare_name: &BareName, value: Variant) -> Result<()> {
        match self.matches_result_name(bare_name) {
            Some(result_name) => {
                // assigning the return value to a function using an unqualified name
                self._do_assign_typed(&result_name, value)
            }
            None => {
                let target_type = self.effective_type_qualifier(bare_name);
                self.set_variable(
                    QualifiedName::new(bare_name.clone(), target_type.clone()),
                    cast(value, target_type.clone())?,
                );
                Ok(())
            }
        }
    }

    fn _assign_typed(&mut self, qualified_name: &QualifiedName, value: Variant) -> Result<()> {
        match self.matches_result_name(&qualified_name.name) {
            Some(result_name) => {
                // assigning the return value to a function using a qualified name
                // make sure the function type matches
                if result_name.qualifier == qualified_name.qualifier {
                    self._do_assign_typed(qualified_name, value)
                } else {
                    self.err("Duplicate definition")
                }
            }
            None => self._do_assign_typed(qualified_name, value),
        }
    }

    fn _do_assign_typed(&mut self, qualified_name: &QualifiedName, value: Variant) -> Result<()> {
        let target_type = qualified_name.qualifier.clone();
        self.set_variable(qualified_name, cast(value, target_type)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::test_utils::*;

    mod assignment {
        use super::*;

        #[test]
        fn test_assign_literal_to_unqualified_float() {
            assert_assign("X").literal("1.0").assert_eq(1.0_f32);
            assert_assign("X").literal("-1.0").assert_eq(-1.0_f32);
            assert_assign("X").literal(".5").assert_eq(0.5_f32);
            assert_assign("X").literal("-.5").assert_eq(-0.5_f32);
            assert_assign("X").literal("1").assert_eq(1.0_f32);
            assert_assign("X").literal("3.14#").assert_eq(3.14_f32);
            assert_assign("X").literal("\"hello\"").assert_err();
        }

        #[test]
        fn test_assign_plus_expression_to_unqualified_float() {
            assert_assign("X")
                .literal(".5 + .5")
                .assert_eq(Variant::from(1.0_f32));
        }

        #[test]
        fn test_assign_literal_to_qualified_float() {
            assert_assign("X!").literal("1.0").assert_eq(1.0_f32);
            assert_assign("X!").literal("1").assert_eq(1.0_f32);
            assert_assign("X!").literal("\"hello\"").assert_err();
        }

        #[test]
        fn test_assign_literal_to_qualified_double() {
            assert_assign("X#")
                .literal("1.0")
                .assert_eq(Variant::from(1.0));
            assert_assign("X#")
                .literal("1")
                .assert_eq(Variant::from(1.0));
            assert_assign("X#")
                .literal("3.14#")
                .assert_eq(Variant::from(3.14));
            assert_assign("X#").literal("\"hello\"").assert_err();
        }

        #[test]
        fn test_assign_literal_to_qualified_string() {
            assert_assign("A$").literal("1.0").assert_err();
            assert_assign("A$").literal("1").assert_err();
            assert_assign("A$").literal("-1").assert_err();
            assert_assign("A$").literal("\"hello\"").assert_eq("hello");
        }

        #[test]
        fn test_assign_literal_to_qualified_integer() {
            assert_assign("X%").literal("1.0").assert_eq(1);
            assert_assign("X%").literal("1.1").assert_eq(1);
            assert_assign("X%").literal("1.5").assert_eq(2);
            assert_assign("X%").literal("1.9").assert_eq(2);
            assert_assign("X%").literal("1").assert_eq(1);
            assert_assign("X%").literal("-1").assert_eq(-1);
            assert_assign("X%").literal("\"hello\"").assert_err();
            assert_assign("X%").literal("3.14#").assert_eq(3);
        }

        #[test]
        fn test_assign_literal_to_qualified_long() {
            assert_assign("X&").literal("1.0").assert_eq(1_i64);
            assert_assign("X&").literal("1.1").assert_eq(1_i64);
            assert_assign("X&").literal("1.5").assert_eq(2_i64);
            assert_assign("X&").literal("1.9").assert_eq(2_i64);
            assert_assign("X&").literal("1").assert_eq(1_i64);
            assert_assign("X&").literal("-1").assert_eq(-1_i64);
            assert_assign("X&").literal("\"hello\"").assert_err();
            assert_assign("X&").literal("3.14#").assert_eq(3_i64);
        }

        #[test]
        fn test_assign_same_variable_name_different_qualifiers() {
            let stdlib = MockStdlib::new();
            let input = "A = 0.1
A# = 3.14
A$ = \"Hello\"
A% = 1
A& = 100";
            let interpreter = interpret(input, stdlib).unwrap();
            interpreter.has_variable("A", 0.1_f32);
            interpreter.has_variable("A!", 0.1_f32);
            interpreter.has_variable_close_enough("A#", 3.14);
            interpreter.has_variable("A$", "Hello");
            interpreter.has_variable("A%", 1);
            interpreter.has_variable("A&", 100_i64);
        }

        #[test]
        fn test_assign_negated_variable() {
            let input = "A = -42
B = -A";
            let interpreter = interpret(input, MockStdlib::new()).unwrap();
            interpreter.has_variable("A", -42.0_f32);
            interpreter.has_variable("B", 42.0_f32);
        }
    }
}
