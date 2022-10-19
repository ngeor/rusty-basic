use crate::common::{FnStateful, OptStateful, QErrorNode, Stateful, Unit};
use crate::linter::converter::expr_rules::ExprStateful;
use crate::linter::converter::statement::on_statements;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::ForLoopNode;

pub fn on_for_loop(
    a: ForLoopNode,
) -> impl Stateful<Output = ForLoopNode, Error = QErrorNode, State = ConverterImpl> {
    let variable_name = ExprStateful::new(a.variable_name, ExprContext::Assignment);
    let lower_bound = ExprStateful::new(a.lower_bound, ExprContext::Default);
    let upper_bound = ExprStateful::new(a.upper_bound, ExprContext::Default);
    let step = Unit::new(a.step).opt_flat_map(|e| ExprStateful::new(e, ExprContext::Default));
    let statements = on_statements(a.statements);
    let next_counter =
        Unit::new(a.next_counter).opt_flat_map(|e| ExprStateful::new(e, ExprContext::Assignment));
    FnStateful::new(move |state: &mut ConverterImpl| {
        Ok(ForLoopNode {
            variable_name: variable_name.unwrap(&mut state.context)?,
            lower_bound: lower_bound.unwrap(&mut state.context)?,
            upper_bound: upper_bound.unwrap(&mut state.context)?,
            step: step.unwrap(&mut state.context)?,
            statements: statements.unwrap(state)?,
            next_counter: next_counter.unwrap(&mut state.context)?,
        })
    })
}
