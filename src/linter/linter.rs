use super::expression_reducer::ExpressionReducer;
use super::post_conversion_linter::PostConversionLinter;
use super::subprogram_context::{FunctionMap, SubMap};
use super::types::*;
use crate::common::*;
use crate::linter::converter;
use crate::parser;

pub fn lint(program: parser::ProgramNode) -> Result<ProgramNode, QErrorNode> {
    // convert to fully typed
    let (result, functions, subs) = converter::convert(program)?;
    // lint
    apply_linters(&result, &functions, &subs)?;
    // reduce
    let reducer = super::undefined_function_reducer::UndefinedFunctionReducer {
        functions: &functions,
    };
    reducer.visit_program(result)
}

fn apply_linters(
    result: &ProgramNode,
    functions: &FunctionMap,
    subs: &SubMap,
) -> Result<(), QErrorNode> {
    let linter = super::no_dynamic_const::NoDynamicConst {};
    linter.visit_program(&result)?;

    let linter = super::for_next_counter_match::ForNextCounterMatch {};
    linter.visit_program(&result)?;

    let linter = super::built_in_linter::BuiltInLinter {};
    linter.visit_program(&result)?;

    let linter = super::user_defined_function_linter::UserDefinedFunctionLinter {
        functions: &functions,
    };
    linter.visit_program(&result)?;

    let linter = super::user_defined_sub_linter::UserDefinedSubLinter { subs: &subs };
    linter.visit_program(&result)?;

    let linter = super::select_case_linter::SelectCaseLinter {};
    linter.visit_program(&result)?;

    let mut linter = super::label_linter::LabelLinter::new();
    linter.visit_program(&result)?;
    linter.switch_to_validating_mode();
    linter.visit_program(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::linter::test_utils::*;
    use crate::linter::*;
    use std::convert::TryInto;

    mod assignment {
        use super::*;

        #[test]
        fn name_clashes_with_other_sub_name() {
            let program = r#"
            SUB Hello
            END SUB
            SUB Oops
            Hello = 2
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 5, 13);
        }

        #[test]
        fn literals_type_mismatch() {
            assert_linter_err!("X = \"hello\"", QError::TypeMismatch, 1, 5);
            assert_linter_err!("X! = \"hello\"", QError::TypeMismatch, 1, 6);
            assert_linter_err!("X# = \"hello\"", QError::TypeMismatch, 1, 6);
            assert_linter_err!("A$ = 1.0", QError::TypeMismatch, 1, 6);
            assert_linter_err!("A$ = 1", QError::TypeMismatch, 1, 6);
            assert_linter_err!("A$ = -1", QError::TypeMismatch, 1, 6);
            assert_linter_err!("X% = \"hello\"", QError::TypeMismatch, 1, 6);
            assert_linter_err!("X& = \"hello\"", QError::TypeMismatch, 1, 6);
        }

        #[test]
        fn assign_to_const() {
            let program = "
            CONST X = 3.14
            X = 6.28
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 13);
        }

        #[test]
        fn assign_to_parent_const() {
            let program = r#"
            CONST X = 42
            SUB Hello
            X = 3
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
        }

        #[test]
        fn assign_integer_to_extended_string() {
            let program = r#"
            X = 1
            IF X = 0 THEN DIM A AS STRING
            A = 42
            "#;
            assert_linter_err!(program, QError::TypeMismatch, 4, 17);
        }
    }

    mod constant {
        use super::*;

        #[test]
        fn function_call_not_allowed() {
            let program = r#"
            CONST X = Add(1, 2)
            "#;
            assert_linter_err!(program, QError::InvalidConstant, 2, 23);
        }

        #[test]
        fn variable_not_allowed() {
            let program = r#"
            X = 42
            CONST A = X + 1
            "#;
            assert_linter_err!(program, QError::InvalidConstant, 3, 23);
        }

        #[test]
        fn variable_already_exists() {
            let program = "
            X = 42
            CONST X = 32
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn variable_already_exists_as_sub_call_param() {
            let program = "
            INPUT X%
            CONST X = 1
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn const_already_exists() {
            let program = "
            CONST X = 32
            CONST X = 33
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn qualified_usage_from_string_literal() {
            let program = r#"
            CONST X! = "hello"
            "#;
            assert_linter_err!(program, QError::TypeMismatch, 2, 24);
        }

        #[test]
        fn const_after_dim_duplicate_definition() {
            let program = r#"
            DIM A AS STRING
            CONST A = "hello"
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn test_global_const_cannot_have_function_name() {
            let program = r#"
            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            CONST GetAction = 42
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 5, 19);
        }

        #[test]
        fn test_local_const_cannot_have_function_name() {
            let program = r#"
            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            FUNCTION Echo(X)
                CONST GetAction = 42
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 6, 23);
        }

        #[test]
        fn test_forward_const_not_allowed() {
            let input = "
            CONST A = B + 1
            CONST B = 42";
            assert_linter_err!(input, QError::InvalidConstant, 2, 23);
        }
    }

    mod dim {
        use super::*;

        #[test]
        fn test_dim_duplicate_definition_same_builtin_type() {
            let program = r#"
            DIM A AS STRING
            DIM A AS STRING
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_duplicate_definition_different_builtin_type() {
            let program = r#"
            DIM A AS STRING
            DIM A AS INTEGER
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_string() {
            let program = r#"
            DIM A AS STRING
            A = "hello"
            PRINT A
            "#;
            assert_eq!(
                linter_ok(program),
                vec![
                    TopLevelToken::Statement(Statement::Dim(
                        DeclaredName::new(
                            "A".into(),
                            TypeDefinition::ExtendedBuiltIn(TypeQualifier::DollarString)
                        )
                        .at_rc(2, 17)
                    ))
                    .at_rc(2, 13),
                    TopLevelToken::Statement(Statement::Assignment(
                        "A$".try_into().unwrap(),
                        Expression::StringLiteral("hello".to_string()).at_rc(3, 17)
                    ))
                    .at_rc(3, 13),
                    TopLevelToken::Statement(Statement::BuiltInSubCall(
                        BuiltInSub::Print,
                        vec![Expression::Variable("A$".try_into().unwrap()).at_rc(4, 19)]
                    ))
                    .at_rc(4, 13)
                ]
            );
        }

        #[test]
        fn test_dim_after_const_duplicate_definition() {
            let program = r#"
            CONST A = "hello"
            DIM A AS STRING
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_after_variable_assignment_duplicate_definition() {
            let program = r#"
            A = 42
            DIM A AS INTEGER
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_string_duplicate_definition() {
            let program = r#"
            DIM A$
            DIM A$
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_bare_duplicate_definition() {
            let program = r#"
            DIM A
            DIM A
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_single_bare_duplicate_definition() {
            // single is the default type
            let program = r#"
            DIM A!
            DIM A
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_bare_single_duplicate_definition() {
            // single is the default type
            let program = r#"
            DIM A
            DIM A!
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_bare_integer_duplicate_definition() {
            let program = r#"
            DEFINT A-Z
            DIM A
            DIM A%
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 17);
        }

        #[test]
        fn test_dim_extended_inside_sub_name_clashing_sub_name() {
            let program = r#"
            SUB Hello
            Dim Hello AS STRING
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_bare_inside_sub_name_clashing_other_sub_name() {
            let program = r#"
            SUB Oops
            END SUB

            SUB Hello
            Dim Oops
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 6, 17);
        }

        #[test]
        fn test_dim_extended_inside_sub_name_clashing_param_name() {
            let program = r#"
            SUB Hello(Oops)
            Dim Oops AS STRING
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_extended_inside_function_name_clashing_function_name() {
            let program = r#"
            FUNCTION Hello
            Dim Hello AS STRING
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_extended_inside_function_name_clashing_other_function_name() {
            let program = r#"
            FUNCTION Hello
            Dim Bar AS STRING
            END FUNCTION
            FUNCTION Bar
            END Function
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }
    }

    mod function_implementation {
        use super::*;

        #[test]
        fn test_function_param_clashing_sub_name_declared_earlier() {
            let program = r#"
            SUB Hello
            END SUB

            FUNCTION Adding(Hello)
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 5, 29);
        }

        #[test]
        fn test_function_param_clashing_sub_name_declared_later() {
            let program = r#"
            FUNCTION Adding(Hello)
            END FUNCTION

            SUB Hello
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 29);
        }

        #[test]
        fn test_function_param_of_different_type_clashing_function_name() {
            let program = r#"
            FUNCTION Adding(Adding$)
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 29);
        }

        #[test]
        fn test_function_param_clashing_function_name_extended_same_type() {
            let program = r#"
            FUNCTION Adding(Adding AS SINGLE)
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 29);
        }

        #[test]
        fn test_function_param_duplicate() {
            let program = r#"
            FUNCTION Adding(Adding, Adding)
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 37);
        }

        #[test]
        fn test_no_args_function_call_cannot_assign_to_variable() {
            let program = r#"
            DECLARE FUNCTION GetAction$

            GetAction% = 42

            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
        }

        #[test]
        fn test_function_call_without_implementation() {
            let program = "
            DECLARE FUNCTION Add(A, B)
            X = Add(1, 2)
            ";
            assert_linter_err!(program, QError::SubprogramNotDefined, 2, 13);
        }

        #[test]
        fn test_cannot_override_built_in_function_with_declaration() {
            let program = r#"
            DECLARE FUNCTION Environ$
            PRINT "Hello"
            FUNCTION Environ$
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
        }

        #[test]
        fn test_cannot_override_built_in_function_without_declaration() {
            let program = r#"
            PRINT "Hello"
            FUNCTION Environ$
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 13);
        }

        #[test]
        fn test_cannot_call_built_in_function_with_wrong_type() {
            let program = r#"
            PRINT "Hello", Environ%("oops")
            "#;
            assert_linter_err!(program, QError::TypeMismatch, 2, 28);
        }

        #[test]
        fn test_function_call_missing_with_string_arguments_gives_type_mismatch() {
            let program = "
            X = Add(\"1\", \"2\")
            ";
            assert_linter_err!(program, QError::ArgumentTypeMismatch, 2, 21);
        }
    }

    mod sub_implementation {
        use super::*;

        #[test]
        fn test_sub_param_clashing_sub_name() {
            let program = r#"
            SUB Hello(Hello)
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 23);
        }

        #[test]
        fn test_sub_param_clashing_other_sub_name_declared_earlier() {
            let program = r#"
            SUB Hello
            END SUB
            SUB Goodbye(Hello)
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 25);
        }

        #[test]
        fn test_sub_param_clashing_other_sub_name_declared_later() {
            let program = r#"
            SUB Goodbye(Hello)
            END SUB
            SUB Hello
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 25);
        }

        #[test]
        fn test_sub_param_duplicate() {
            let program = r#"
            SUB Hello(A, A)
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 26);
        }

        #[test]
        fn test_sub_param_extended_duplicate() {
            let program = r#"
            SUB Hello(A AS INTEGER, A AS STRING)
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 37);
        }

        #[test]
        fn test_cannot_override_built_in_sub_with_declaration() {
            let program = r#"
            DECLARE SUB Environ
            PRINT "Hello"
            SUB Environ
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
        }

        #[test]
        fn test_cannot_override_built_in_sub_without_declaration() {
            let program = r#"
            PRINT "Hello"
            SUB Environ
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 13);
        }

        #[test]
        fn test_by_ref_parameter_type_mismatch() {
            let program = "
            DECLARE SUB Hello(N)
            A% = 42
            Hello A%
            SUB Hello(N)
                N = N + 1
            END SUB
            ";
            assert_linter_err!(program, QError::ArgumentTypeMismatch, 4, 19);
        }
    }

    mod select_case {
        use super::*;

        #[test]
        fn test_select_wrong_type_in_simple_case() {
            let input = r#"
        SELECT CASE 42
            CASE "book"
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 18);
        }

        #[test]
        fn test_select_wrong_type_in_range_case_upper() {
            let input = r#"
        SELECT CASE 42
            CASE 1 TO "book"
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 23);
        }

        #[test]
        fn test_select_wrong_type_in_range_case_lower() {
            let input = r#"
        SELECT CASE 42
            CASE "abc" TO 12
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 18);
        }

        #[test]
        fn test_select_wrong_type_in_range_case_both() {
            let input = r#"
        SELECT CASE 42
            CASE "abc" TO "def"
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 18);
        }

        #[test]
        fn test_select_wrong_type_in_is() {
            let input = r#"
        SELECT CASE 42
            CASE IS >= "abc"
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 24);
        }
    }

    mod go_to {
        use super::*;

        #[test]
        fn on_error_go_to_missing_label() {
            let input = r#"
            ON ERROR GOTO ErrTrap
            "#;
            assert_linter_err!(input, QError::LabelNotDefined, 2, 13);
        }

        #[test]
        fn go_to_missing_label() {
            let input = "
            GOTO Jump
            ";
            assert_linter_err!(input, QError::LabelNotDefined, 2, 13);
        }

        #[test]
        fn go_to_duplicate_label() {
            let input = "
            GOTO Jump
            Jump:
            Jump:
            ";
            assert_linter_err!(input, QError::DuplicateLabel, 4, 13);
        }
    }

    mod for_loop {
        use super::*;

        #[test]
        fn test_for_loop_with_wrong_next_counter() {
            let input = "
            FOR i% = 1 TO 5
                PRINT i%
            NEXT i
            ";
            assert_linter_err!(input, QError::NextWithoutFor, 4, 18);
        }
    }

    mod expression {
        use super::*;

        macro_rules! assert_condition_err {
            ($condition:expr, $col:expr) => {
                let program = format!(
                    "
                IF {} THEN
                    PRINT \"hi\"
                END IF
                ",
                    $condition
                );
                assert_linter_err!(program, QError::TypeMismatch, 2, $col);
            };
        }

        #[test]
        fn test_type_mismatch() {
            assert_linter_err!("X = 1.1 + \"hello\"", QError::TypeMismatch, 1, 11);
            assert_linter_err!("X = 1.1# + \"hello\"", QError::TypeMismatch, 1, 12);
            assert_linter_err!("X$ = \"hello\" + 1", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" + 1.1", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" + 1.1#", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X% = 1 + \"hello\"", QError::TypeMismatch, 1, 10);
            assert_linter_err!("X& = 1 + \"hello\"", QError::TypeMismatch, 1, 10);
            assert_linter_err!("X = 1.1 - \"hello\"", QError::TypeMismatch, 1, 11);
            assert_linter_err!("X = 1.1# - \"hello\"", QError::TypeMismatch, 1, 12);
            assert_linter_err!("X$ = \"hello\" - \"hi\"", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" - 1", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" - 1.1", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" - 1.1#", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = 1 - \"hello\"", QError::TypeMismatch, 1, 10);
            assert_linter_err!("X& = 1 - \"hello\"", QError::TypeMismatch, 1, 10);
            assert_linter_err!(r#"PRINT "hello" * 5"#, QError::TypeMismatch, 1, 17);
            assert_linter_err!(r#"PRINT "hello" / 5"#, QError::TypeMismatch, 1, 17);
            assert_linter_err!("X = -\"hello\"", QError::TypeMismatch, 1, 6);
            assert_linter_err!("X% = -\"hello\"", QError::TypeMismatch, 1, 7);
            assert_linter_err!("X = NOT \"hello\"", QError::TypeMismatch, 1, 9);
            assert_linter_err!("X% = NOT \"hello\"", QError::TypeMismatch, 1, 10);

            assert_linter_err!(r#"PRINT 1 AND "hello""#, QError::TypeMismatch, 1, 13);
            assert_linter_err!(r#"PRINT "hello" AND 1"#, QError::TypeMismatch, 1, 19);
            assert_linter_err!(r#"PRINT "hello" AND "bye""#, QError::TypeMismatch, 1, 19);

            assert_linter_err!(r#"PRINT 1 AND #1"#, QError::TypeMismatch, 1, 13);
            assert_linter_err!(r#"PRINT #1 AND 1"#, QError::TypeMismatch, 1, 7);
            assert_linter_err!(r#"PRINT #1 AND #1"#, QError::TypeMismatch, 1, 7);
        }

        #[test]
        fn test_condition_type_mismatch() {
            assert_condition_err!("9.1 < \"hello\"", 26);
            assert_condition_err!("9.1# < \"hello\"", 27);
            assert_condition_err!("\"hello\" < 3.14", 30);
            assert_condition_err!("\"hello\" < 3", 30);
            assert_condition_err!("\"hello\" < 3.14#", 30);
            assert_condition_err!("9 < \"hello\"", 24);
            assert_condition_err!("9.1 <= \"hello\"", 27);
            assert_condition_err!("9.1# <= \"hello\"", 28);
            assert_condition_err!("\"hello\" <= 3.14", 31);
            assert_condition_err!("\"hello\" <= 3", 31);
            assert_condition_err!("\"hello\" <= 3.14#", 31);
            assert_condition_err!("9 <= \"hello\"", 25);
        }

        #[test]
        fn qualified_const_usage_wrong_type() {
            let program = "
            CONST X = 42
            PRINT X!
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }
    }

    mod user_defined_type {
        use super::*;

        #[test]
        fn duplicate_type_throws_duplicate_definition() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            TYPE Card
                Value AS INTEGER
            END TYPE";
            assert_linter_err!(input, QError::DuplicateDefinition, 6, 13);
        }

        #[test]
        fn duplicate_element_name() {
            let input = "
            TYPE Card
                Value AS INTEGER
                Value AS INTEGER
            END TYPE
        ";
            assert_linter_err!(input, QError::DuplicateDefinition, 4, 17);
        }

        #[test]
        fn element_using_container_type_throws_type_not_defined() {
            let input = "
            TYPE Card
                Item AS Card
            END TYPE";
            // QBasic actually positions the error on the "AS" keyword
            assert_linter_err!(input, QError::syntax_error("Type not defined"), 3, 25);
        }

        #[test]
        fn using_type_before_defined_throws_type_not_defined() {
            let input = "
            TYPE Address
                PostCode AS PostCode
            END TYPE

            TYPE PostCode
                Prefix AS INTEGER
                Suffix AS STRING * 2
            END TYPE";
            assert_linter_err!(input, QError::syntax_error("Type not defined"), 3, 29);
        }

        #[test]
        fn string_length_must_be_constant() {
            let input = "
            TYPE Invalid
                N AS STRING * A
            END TYPE";
            assert_linter_err!(input, QError::InvalidConstant, 3, 31);
        }

        #[test]
        fn string_length_must_be_constant_const_cannot_follow_type() {
            let input = "
            TYPE Invalid
                N AS STRING * A
            END TYPE

            CONST A = 10";
            assert_linter_err!(input, QError::InvalidConstant, 3, 31);
        }

        #[test]
        fn type_mismatch_when_used_in_print() {
            let input = "
            TYPE Card
                Suit AS STRING * 9
                Value AS INTEGER
            END TYPE

            DIM c AS Card
            PRINT c";
            assert_linter_err!(input, QError::TypeMismatch, 8, 19);
        }

        #[test]
        fn referencing_non_existing_member() {
            let input = "
            TYPE Card
                Suit AS STRING * 9
                Value AS INTEGER
            END TYPE

            DIM c AS Card
            PRINT c.Suite";
            assert_linter_err!(input, QError::syntax_error("Element not defined"), 8, 20);
        }

        #[test]
        fn cannot_use_in_binary_expression() {
            let ops = [
                "=", "<>", ">=", ">", "<", "<=", "+", "-", "*", "/", "AND", "OR",
            ];
            for op in &ops {
                let input = format!(
                    "
                    TYPE Card
                        Value AS INTEGER
                    END TYPE

                    DIM a AS CARD
                    DIM b AS CARD

                    IF a {} b THEN
                    END IF",
                    op
                );
                assert_linter_err!(input, QError::TypeMismatch, 9, 26);
            }
        }

        #[test]
        fn cannot_use_in_unary_expression() {
            let ops = ["-", "NOT "];
            for op in &ops {
                let input = format!(
                    "
                    TYPE Card
                        Value AS INTEGER
                    END TYPE

                    DIM a AS CARD
                    DIM b AS CARD

                    b = {}A",
                    op
                );
                assert_linter_err!(input, QError::TypeMismatch, 9, 25);
            }
        }
    }
}
// TODO test file handle expression cannot be used anywhere except for `OPEN`, `CLOSE`, `LINE INPUT`, `INPUT`
