use crate::common::{FnStateful, OptStateful, QErrorNode, Stateful, Unit};
use crate::linter::converter::expr_rules::ExprStateful;
use crate::linter::converter::statement::StatementsRemovingConstantsStateful;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::ForLoopNode;

pub fn on_for_loop(
    a: ForLoopNode,
) -> impl Stateful<Output = ForLoopNode, Error = QErrorNode, State = ConverterImpl> {
    let variable_name = ExprStateful::new(a.variable_name, ExprContext::Assignment);
    let lower_bound = ExprStateful::new(a.lower_bound, ExprContext::Default);
    let upper_bound = ExprStateful::new(a.upper_bound, ExprContext::Default);
    let step = Unit::new(a.step).opt_flat_map(|e| ExprStateful::new(e, ExprContext::Default));
    let statements = StatementsRemovingConstantsStateful::new(a.statements);
    let next_counter =
        Unit::new(a.next_counter).opt_flat_map(|e| ExprStateful::new(e, ExprContext::Assignment));
    FnStateful::new(move |state: &mut ConverterImpl| {
        Ok(ForLoopNode {
            variable_name: variable_name.unwrap(state)?,
            lower_bound: lower_bound.unwrap(state)?,
            upper_bound: upper_bound.unwrap(state)?,
            step: step.unwrap(state)?,
            statements: statements.unwrap(state)?,
            next_counter: next_counter.unwrap(state)?,
        })
    })
}
