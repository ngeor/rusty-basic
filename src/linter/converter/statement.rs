use crate::common::*;
use crate::linter::converter::converter::Context;
use crate::linter::converter::do_loop::on_do_loop;
use crate::linter::converter::for_loop::on_for_loop;
use crate::linter::converter::if_blocks::{on_conditional_block, on_if_block};
use crate::linter::converter::print_node::on_print_node;
use crate::linter::converter::select_case::on_select_case;
use crate::linter::converter::ExprContext;
use crate::linter::{DimContext, NameContext};
use crate::parser::{
    BareName, ExitObject, Expression, Name, Statement, StatementNode, StatementNodes,
};

pub struct StatementStateful(StatementNode);

impl StatementStateful {
    pub fn new(statement_node: StatementNode) -> Self {
        Self(statement_node)
    }
}

impl Stateful for StatementStateful {
    type Output = StatementNode;
    type State = Context;
    type Error = QErrorNode;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let Locatable {
            element: statement,
            pos,
        } = self.0;
        match statement {
            Statement::Assignment(n, e) => state.assignment(n.at(pos), e),
            // CONST will be filtered out in the StatementNodes processor
            Statement::Const(n, e) => state.on_const(n, e).map(|_| dummy_const()),
            Statement::SubCall(n, args) => state.sub_call(n.at(pos), args),
            Statement::BuiltInSubCall(built_in_sub, args) => {
                let converted_args = state.on_expressions(args, ExprContext::Parameter)?;
                Ok(Statement::BuiltInSubCall(built_in_sub, converted_args).at(pos))
            }
            Statement::IfBlock(i) => on_if_block(i)
                .map(|i| Statement::IfBlock(i).at(pos))
                .unwrap(state),
            Statement::SelectCase(s) => on_select_case(s)
                .map(|s| Statement::SelectCase(s).at(pos))
                .unwrap(state),
            Statement::ForLoop(f) => on_for_loop(f)
                .map(|f| Statement::ForLoop(f).at(pos))
                .unwrap(state),
            Statement::While(c) => on_conditional_block(c)
                .map(|c| Statement::While(c).at(pos))
                .unwrap(state),
            Statement::DoLoop(do_loop_node) => on_do_loop(do_loop_node)
                .map(|d| Statement::DoLoop(d).at(pos))
                .unwrap(state),
            Statement::Return(opt_label) => {
                if opt_label.is_some() && state.is_in_subprogram() {
                    // cannot have RETURN with explicit label inside subprogram
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
                } else {
                    Ok(Statement::Return(opt_label).at(pos))
                }
            }
            Statement::Resume(resume_option) => {
                if state.is_in_subprogram() {
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
                } else {
                    Ok(Statement::Resume(resume_option).at(pos))
                }
            }
            Statement::Exit(exit_object) => match state.get_name_context() {
                NameContext::Global => {
                    Err(QError::syntax_error("Illegal outside of subprogram")).with_err_at(pos)
                }
                NameContext::Sub => {
                    if exit_object == ExitObject::Sub {
                        Ok(Statement::Exit(exit_object).at(pos))
                    } else {
                        Err(QError::syntax_error("Illegal inside sub")).with_err_at(pos)
                    }
                }
                NameContext::Function => {
                    if exit_object == ExitObject::Function {
                        Ok(Statement::Exit(exit_object).at(pos))
                    } else {
                        Err(QError::syntax_error("Illegal inside function")).with_err_at(pos)
                    }
                }
            },
            Statement::Dim(dim_list) => state
                .on_dim(dim_list, DimContext::Default)
                .map(|dim_list| Statement::Dim(dim_list).at(pos)),
            Statement::Redim(dim_list) => state
                .on_dim(dim_list, DimContext::Redim)
                .map(|dim_list| Statement::Redim(dim_list).at(pos)),
            Statement::Print(print_node) => on_print_node(print_node)
                .map(Statement::Print)
                .map(|s| s.at(pos))
                .unwrap(state),
            Statement::OnError(_)
            | Statement::Label(_)
            | Statement::GoTo(_)
            | Statement::GoSub(_)
            | Statement::Comment(_)
            | Statement::End
            | Statement::System => Ok(statement.at(pos)),
        }
    }
}

fn dummy_const() -> StatementNode {
    let pos = Location::start();
    Statement::Const(
        Name::Bare(BareName::new(String::new())).at(pos),
        Expression::IntegerLiteral(0).at(pos),
    )
    .at(pos)
}

pub fn on_statements(
    statements: StatementNodes,
) -> impl Stateful<Output = StatementNodes, Error = QErrorNode, State = Context> {
    Unit::new(statements)
        .vec_flat_map(|s| StatementStateful::new(s))
        // filter out CONST statements, they've been registered into context as values
        .vec_filter(|s| {
            if let Statement::Const(_, _) = s.as_ref() {
                false
            } else {
                true
            }
        })
}
