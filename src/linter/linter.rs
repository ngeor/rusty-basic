use super::error::*;
use super::expression_reducer::ExpressionReducer;
use super::post_conversion_linter::PostConversionLinter;
use super::subprogram_context::{FunctionMap, SubMap};
use super::types::*;
use crate::linter::converter;
use crate::parser;

pub fn lint(program: parser::ProgramNode) -> Result<ProgramNode, Error> {
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
) -> Result<(), Error> {
    let linter = super::no_dynamic_const::NoDynamicConst {};
    linter.visit_program(&result)?;

    let linter = super::for_next_counter_match::ForNextCounterMatch {};
    linter.visit_program(&result)?;

    let linter = super::built_in_function_linter::BuiltInFunctionLinter {};
    linter.visit_program(&result)?;

    let linter = super::built_in_sub_linter::BuiltInSubLinter {};
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
    use crate::linter::*;

    mod dim {
        use super::*;

        #[test]
        fn test_dim_type_mismatch() {
            let program = r#"
            X = 1
            IF X = 0 THEN DIM A AS STRING
            A = 42
            "#;
            assert_linter_err!(program, LinterError::TypeMismatch, 4, 17);
        }

        #[test]
        fn test_dim_duplicate_definition_same_builtin_type() {
            let program = r#"
            DIM A AS STRING
            DIM A AS STRING
            "#;
            assert_linter_err!(program, LinterError::DuplicateDefinition, 3, 13);
        }

        #[test]
        fn test_dim_duplicate_definition_different_builtin_type() {
            let program = r#"
            DIM A AS STRING
            DIM A AS INTEGER
            "#;
            assert_linter_err!(program, LinterError::DuplicateDefinition, 3, 13);
        }
    }
}
