use crate::common::{
    FnStateful, InChildState, OptStateful, QErrorNode, Stateful, Unit, VecStateful,
};
use crate::linter::converter::converter::Context;
use crate::linter::converter::expr_rules::ExprStateful;
use crate::linter::converter::statement::on_statements;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::{CaseBlockNode, CaseExpression, SelectCaseNode};

pub fn on_select_case(
    a: SelectCaseNode,
) -> impl Stateful<Output = SelectCaseNode, Error = QErrorNode, State = ConverterImpl> {
    let expr = ExprStateful::new(a.expr, ExprContext::Default);
    let case_blocks = Unit::new(a.case_blocks).vec_flat_map(on_case_block);
    let else_block = Unit::new(a.else_block).opt_flat_map(on_statements);
    let inline_comments = a.inline_comments;
    FnStateful::new(move |ctx: &mut ConverterImpl| {
        Ok(SelectCaseNode {
            expr: expr.unwrap(&mut ctx.context)?,
            case_blocks: case_blocks.unwrap(ctx)?,
            else_block: else_block.unwrap(ctx)?,
            inline_comments,
        })
    })
}

fn on_case_block(
    a: CaseBlockNode,
) -> impl Stateful<Output = CaseBlockNode, Error = QErrorNode, State = ConverterImpl> {
    let expression_list = InChildState::new(
        Unit::new(a.expression_list).vec_flat_map(CaseExpressionStateful),
        ConverterImpl::context,
    );
    let statements = on_statements(a.statements);
    (expression_list, statements).map(|(expression_list, statements)| CaseBlockNode {
        expression_list,
        statements,
    })
}

struct CaseExpressionStateful(CaseExpression);

impl Stateful for CaseExpressionStateful {
    type Output = CaseExpression;
    type State = Context;
    type Error = QErrorNode;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        // TODO: (low prio) needs explicit implementation for match due to arms having different types (different closures are different types)
        match self.0 {
            CaseExpression::Simple(e) => ExprStateful::new(e, ExprContext::Default)
                .map(|expr| CaseExpression::Simple(expr))
                .unwrap(state),
            CaseExpression::Is(op, e) => ExprStateful::new(e, ExprContext::Default)
                .map(|expr| CaseExpression::Is(op, expr))
                .unwrap(state),
            CaseExpression::Range(from, to) => {
                let from = ExprStateful::new(from, ExprContext::Default);
                let to = ExprStateful::new(to, ExprContext::Default);
                (from, to)
                    .map(|(from, to)| CaseExpression::Range(from, to))
                    .unwrap(state)
            }
        }
    }
}
