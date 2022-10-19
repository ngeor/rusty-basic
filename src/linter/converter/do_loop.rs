use crate::common::{QErrorNode, Stateful};
use crate::linter::converter::expr_rules::ExprStateful;
use crate::linter::converter::statement::on_statements;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::DoLoopNode;

pub fn on_do_loop(
    do_loop_node: DoLoopNode,
) -> impl Stateful<Output = DoLoopNode, Error = QErrorNode, State = ConverterImpl> {
    let DoLoopNode {
        condition,
        statements,
        position,
        kind,
    } = do_loop_node;
    let condition_stateful =
        ExprStateful::new(condition, ExprContext::Default).in_child_state(ConverterImpl::context);
    let statements_stateful = on_statements(statements);
    let pair = (condition_stateful, statements_stateful);
    pair.map(move |(condition, statements)| DoLoopNode {
        condition,
        statements,
        position,
        kind,
    })
}
