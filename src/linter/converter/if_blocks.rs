use crate::common::{FnStateful, OptStateful, QErrorNode, Stateful, Unit, VecStateful};
use crate::linter::converter::expr_rules::ExprStateful;
use crate::linter::converter::statement::StatementsRemovingConstantsStateful;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::{ConditionalBlockNode, IfBlockNode};

pub fn on_conditional_block(
    a: ConditionalBlockNode,
) -> impl Stateful<Output = ConditionalBlockNode, Error = QErrorNode, State = ConverterImpl> {
    let condition = ExprStateful::new(a.condition, ExprContext::Default);
    let statements = StatementsRemovingConstantsStateful::new(a.statements);
    (condition, statements).map(|(condition, statements)| ConditionalBlockNode {
        condition,
        statements,
    })
}

pub fn on_if_block(
    a: IfBlockNode,
) -> impl Stateful<Output = IfBlockNode, Error = QErrorNode, State = ConverterImpl> {
    let if_block = on_conditional_block(a.if_block);
    let else_if_blocks = Unit::new(a.else_if_blocks).vec_flat_map(on_conditional_block);
    let else_block = Unit::new(a.else_block).opt_flat_map(StatementsRemovingConstantsStateful::new);
    FnStateful::new(move |ctx: &mut ConverterImpl| {
        Ok(IfBlockNode {
            if_block: if_block.unwrap(ctx)?,
            else_if_blocks: else_if_blocks.unwrap(ctx)?,
            else_block: else_block.unwrap(ctx)?,
        })
    })
}
