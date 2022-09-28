use crate::common::{CaseInsensitiveString, QErrorNode};
use crate::linter::post_linter::expression_reducer::ExpressionReducer;
use crate::linter::post_linter::post_conversion_linter::PostConversionLinter;
use crate::linter::post_linter::{
    built_in_linter, condition_type_linter, dots_linter, for_next_counter_match_linter,
    label_linter, print_linter, select_case_linter, undefined_function_reducer,
    user_defined_function_linter, user_defined_sub_linter,
};
use crate::parser::{FunctionMap, ProgramNode, SubMap};
use std::collections::HashSet;

pub fn post_linter(
    result: ProgramNode,
    functions: &FunctionMap,
    subs: &SubMap,
    names_without_dot: &HashSet<CaseInsensitiveString>,
) -> Result<ProgramNode, QErrorNode> {
    // lint
    apply_linters(&result, functions, subs, names_without_dot)?;
    // reduce
    let mut reducer = undefined_function_reducer::UndefinedFunctionReducer { functions };
    reducer.visit_program(result)
}

fn apply_linters(
    result: &ProgramNode,
    functions: &FunctionMap,
    subs: &SubMap,
    names_without_dot: &HashSet<CaseInsensitiveString>,
) -> Result<(), QErrorNode> {
    let mut linter = for_next_counter_match_linter::ForNextCounterMatch {};
    linter.visit_program(&result)?;

    let mut linter = dots_linter::DotsLinter { names_without_dot };
    linter.visit_program(&result)?;

    let mut linter = built_in_linter::BuiltInLinter::new();
    linter.visit_program(&result)?;

    let mut linter = print_linter::PrintLinter {};
    linter.visit_program(&result)?;

    let mut linter = user_defined_function_linter::UserDefinedFunctionLinter { functions };
    linter.visit_program(&result)?;

    let mut linter = user_defined_sub_linter::UserDefinedSubLinter { subs };
    linter.visit_program(&result)?;

    let mut linter = select_case_linter::SelectCaseLinter {};
    linter.visit_program(&result)?;

    let mut linter = condition_type_linter::ConditionTypeLinter {};
    linter.visit_program(&result)?;

    let mut linter = label_linter::LabelLinter::default();
    linter.visit_program(&result)?;
    Ok(())
}
