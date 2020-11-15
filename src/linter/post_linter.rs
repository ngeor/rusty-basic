pub mod built_in_linter;
pub mod const_reducer;
pub mod dots_linter;
pub mod expression_reducer;
pub mod for_next_counter_match_linter;
pub mod label_linter;
pub mod post_conversion_linter;
pub mod print_linter;
pub mod select_case_linter;
pub mod undefined_function_reducer;
pub mod user_defined_function_linter;
pub mod user_defined_sub_linter;

use crate::common::{CaseInsensitiveString, QErrorNode};
use crate::linter::post_linter::expression_reducer::ExpressionReducer;
use crate::linter::post_linter::post_conversion_linter::PostConversionLinter;
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
    let mut reducer = const_reducer::ConstReducer::new();
    let result = reducer.visit_program(result)?;
    let mut reducer = undefined_function_reducer::UndefinedFunctionReducer { functions };
    reducer.visit_program(result)
}

fn apply_linters(
    result: &ProgramNode,
    functions: &FunctionMap,
    subs: &SubMap,
    names_without_dot: &HashSet<CaseInsensitiveString>,
) -> Result<(), QErrorNode> {
    let linter = for_next_counter_match_linter::ForNextCounterMatch {};
    linter.visit_program(&result)?;

    let linter = dots_linter::DotsLinter { names_without_dot };
    linter.visit_program(&result)?;

    let linter = built_in_linter::BuiltInLinter {};
    linter.visit_program(&result)?;

    let linter = print_linter::PrintLinter {};
    linter.visit_program(&result)?;

    let linter = user_defined_function_linter::UserDefinedFunctionLinter { functions };
    linter.visit_program(&result)?;

    let linter = user_defined_sub_linter::UserDefinedSubLinter { subs };
    linter.visit_program(&result)?;

    let linter = select_case_linter::SelectCaseLinter {};
    linter.visit_program(&result)?;

    let mut linter = label_linter::LabelLinter::new();
    linter.visit_program(&result)?;
    linter.switch_to_validating_mode();
    linter.visit_program(&result)?;
    Ok(())
}
