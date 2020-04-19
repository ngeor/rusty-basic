use super::variable_setter::VariableSetter;
use super::{Interpreter, Result, Stdlib, Variant};
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
    use crate::assert_has_variable;
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
            let input = "A = 0.1
A# = 3.14
A$ = \"Hello\"
A% = 1
A& = 100";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A", 0.1_f32);
            assert_has_variable!(interpreter, "A!", 0.1_f32);
            assert_has_variable!(interpreter, "A#", 3.14);
            assert_has_variable!(interpreter, "A$", "Hello");
            assert_has_variable!(interpreter, "A%", 1);
            assert_has_variable!(interpreter, "A&", 100_i64);
        }

        #[test]
        fn test_assign_negated_variable() {
            let input = "A = -42
B = -A";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A", -42.0_f32);
            assert_has_variable!(interpreter, "B", 42.0_f32);
        }

        #[test]
        fn test_assign_variable_bare_lower_case() {
            let input = "
            A = 42
            b = 12
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A", 42.0_f32);
            assert_has_variable!(interpreter, "a", 42.0_f32);
            assert_has_variable!(interpreter, "B", 12.0_f32);
            assert_has_variable!(interpreter, "b", 12.0_f32);
        }

        #[test]
        fn test_assign_variable_typed_lower_case() {
            let input = "
            A% = 42
            b% = 12
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A%", 42);
            assert_has_variable!(interpreter, "a%", 42);
            assert_has_variable!(interpreter, "B%", 12);
            assert_has_variable!(interpreter, "b%", 12);
        }

        #[test]
        fn test_increment_variable_bare_lower_case() {
            let input = "
            A = 42
            A = a + 1
            b = 12
            B = b + 1
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A", 43_f32);
            assert_has_variable!(interpreter, "a", 43_f32);
            assert_has_variable!(interpreter, "B", 13_f32);
            assert_has_variable!(interpreter, "b", 13_f32);
        }

        #[test]
        fn test_increment_variable_typed_lower_case() {
            let input = "
            A% = 42
            A% = a% + 1
            b% = 12
            B% = b% + 1
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A%", 43);
            assert_has_variable!(interpreter, "a%", 43);
            assert_has_variable!(interpreter, "B%", 13);
            assert_has_variable!(interpreter, "b%", 13);
        }

        #[test]
        fn test_assign_with_def_dbl() {
            let input = "
            DEFDBL A-Z
            A = 6.28
            A! = 3.14
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A", 6.28_f64);
            assert_has_variable!(interpreter, "A!", 3.14_f32);
            assert_has_variable!(interpreter, "A#", 6.28_f64);
        }

        #[test]
        fn test_assign_with_def_int() {
            let input = "
            DEFINT A-Z
            A = 42
            A! = 3.14
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A", 42);
            assert_has_variable!(interpreter, "A!", 3.14_f32);
            assert_has_variable!(interpreter, "A%", 42);
        }

        #[test]
        fn test_assign_with_def_lng() {
            let input = "
            DEFLNG A-Z
            A = 42
            A! = 3.14
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A", 42_i64);
            assert_has_variable!(interpreter, "A!", 3.14_f32);
            assert_has_variable!(interpreter, "A&", 42_i64);
        }

        #[test]
        fn test_assign_with_def_sng() {
            let input = "
            DEFSNG A-Z
            A = 42
            A! = 3.14
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A", 3.14_f32);
            assert_has_variable!(interpreter, "A!", 3.14_f32);
        }

        #[test]
        fn test_assign_with_def_str() {
            let input = r#"
            DEFSTR A-Z
            A = "hello"
            A! = 3.14
            "#;
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A", "hello");
            assert_has_variable!(interpreter, "A!", 3.14_f32);
            assert_has_variable!(interpreter, "A$", "hello");
        }
    }
}
