use super::{Interpreter, Result, Stdlib, VariableSetter, Variant};
use crate::parser::{ExpressionNode, NameNode};

impl<S: Stdlib> Interpreter<S> {
    pub fn assignment(
        &mut self,
        left_side: &NameNode,
        right_side: &ExpressionNode,
    ) -> Result<Option<Variant>> {
        let val: Variant = self.evaluate_expression(right_side)?;
        self.set_variable(left_side, val)
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

        #[test]
        fn test_assign_variable_bare_lower_case() {
            let input = "
            A = 42
            b = 12
            ";
            let interpreter = interpret(input, MockStdlib::new()).unwrap();
            interpreter.has_variable("A", 42.0_f32);
            interpreter.has_variable("a", 42.0_f32);
            interpreter.has_variable("B", 12.0_f32);
            interpreter.has_variable("b", 12.0_f32);
        }

        #[test]
        fn test_assign_variable_typed_lower_case() {
            let input = "
            A% = 42
            b% = 12
            ";
            let interpreter = interpret(input, MockStdlib::new()).unwrap();
            interpreter.has_variable("A%", 42);
            interpreter.has_variable("a%", 42);
            interpreter.has_variable("B%", 12);
            interpreter.has_variable("b%", 12);
        }

        #[test]
        fn test_increment_variable_bare_lower_case() {
            let input = "
            A = 42
            A = a + 1
            b = 12
            B = b + 1
            ";
            let interpreter = interpret(input, MockStdlib::new()).unwrap();
            interpreter.has_variable("A", 43_f32);
            interpreter.has_variable("a", 43_f32);
            interpreter.has_variable("B", 13_f32);
            interpreter.has_variable("b", 13_f32);
        }

        #[test]
        fn test_increment_variable_typed_lower_case() {
            let input = "
            A% = 42
            A% = a% + 1
            b% = 12
            B% = b% + 1
            ";
            let interpreter = interpret(input, MockStdlib::new()).unwrap();
            interpreter.has_variable("A%", 43);
            interpreter.has_variable("a%", 43);
            interpreter.has_variable("B%", 13);
            interpreter.has_variable("b%", 13);
        }
    }
}
