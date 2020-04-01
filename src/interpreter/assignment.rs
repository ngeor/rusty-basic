use super::casting::cast;
use super::context::ReadWriteContext;
use super::*;
use crate::common::Result;
use crate::parser::*;
use std::io::BufRead;

impl<T: BufRead, S: Stdlib> Interpreter<T, S> {
    pub fn assignment(&mut self, left_side: &QName, right_side: &Expression) -> Result<()> {
        let val = self.evaluate_expression(right_side)?;
        let target_type = self.effective_type_qualifier(left_side);
        self.set_variable(left_side, cast(val, target_type)?)
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
            let interpreter = interpret(input, stdlib);
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
            let interpreter = interpret(input, MockStdlib::new());
            interpreter.has_variable("A", -42.0_f32);
            interpreter.has_variable("B", 42.0_f32);
        }
    }
}
