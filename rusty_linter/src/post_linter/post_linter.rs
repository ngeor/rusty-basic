use crate::post_linter::expression_reducer::ExpressionReducer;
use crate::post_linter::post_conversion_linter::PostConversionLinter;
use crate::post_linter::{
    built_in_linter, condition_type_linter, dots_linter, for_next_counter_match_linter,
    label_linter, print_linter, select_case_linter, undefined_function_reducer,
    user_defined_function_linter, user_defined_sub_linter,
};
use crate::{HasFunctions, HasSubs};
use rusty_common::QErrorPos;
use rusty_parser::Program;

pub fn post_linter(
    result: Program,
    pre_linter_result: &(impl HasFunctions + HasSubs),
) -> Result<Program, QErrorPos> {
    // lint
    apply_linters(&result, pre_linter_result)?;
    // reduce
    let mut reducer = undefined_function_reducer::UndefinedFunctionReducer {
        context: pre_linter_result,
    };
    reducer.visit_program(result)
}

fn apply_linters(
    result: &Program,
    pre_linter_result: &(impl HasFunctions + HasSubs),
) -> Result<(), QErrorPos> {
    let mut linter = for_next_counter_match_linter::ForNextCounterMatch {};
    linter.visit_program(result)?;

    let mut linter = dots_linter::DotsLinter::default();
    linter.visit_program(result)?;

    let mut linter = built_in_linter::BuiltInLinter::new();
    linter.visit_program(result)?;

    let mut linter = print_linter::PrintLinter {};
    linter.visit_program(result)?;

    let mut linter = user_defined_function_linter::UserDefinedFunctionLinter {
        context: pre_linter_result,
    };
    linter.visit_program(result)?;

    let mut linter = user_defined_sub_linter::UserDefinedSubLinter {
        context: pre_linter_result,
    };
    linter.visit_program(result)?;

    let mut linter = select_case_linter::SelectCaseLinter {};
    linter.visit_program(result)?;

    let mut linter = condition_type_linter::ConditionTypeLinter {};
    linter.visit_program(result)?;

    let mut linter = label_linter::LabelLinter::default();
    linter.visit_program(result)?;
    Ok(())
}
