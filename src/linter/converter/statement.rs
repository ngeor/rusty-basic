use crate::common::*;
use crate::linter::converter::conversion_traits::SameTypeConverter;
use crate::linter::converter::do_loop::on_do_loop;
use crate::linter::converter::for_loop::on_for_loop;
use crate::linter::converter::if_blocks::{on_conditional_block, on_if_block};
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::linter::{DimContext, NameContext};
use crate::parser::{
    BareName, ExitObject, Expression, Name, Statement, StatementNode, StatementNodes,
};

impl SameTypeConverter<Option<StatementNodes>> for ConverterImpl {
    fn convert(
        &mut self,
        item: Option<StatementNodes>,
    ) -> Result<Option<StatementNodes>, QErrorNode> {
        match item {
            Some(statements) => {
                let converted_statements = self.convert_block_removing_constants(statements)?;
                Ok(Some(converted_statements))
            }
            None => Ok(None),
        }
    }
}

impl SameTypeConverter<StatementNode> for ConverterImpl {
    fn convert(&mut self, statement_node: StatementNode) -> Result<StatementNode, QErrorNode> {
        let Locatable {
            element: statement,
            pos,
        } = statement_node;
        match statement {
            Statement::Assignment(n, e) => self.assignment(n.at(pos), e),
            // CONST will be filtered out in the StatementNodes processor
            Statement::Const(n, e) => self.context.on_const(n, e).map(|_| dummy_const()),
            Statement::SubCall(n, args) => self.sub_call(n.at(pos), args),
            Statement::BuiltInSubCall(built_in_sub, args) => {
                let converted_args = self.context.on_expressions(args, ExprContext::Parameter)?;
                Ok(Statement::BuiltInSubCall(built_in_sub, converted_args).at(pos))
            }
            Statement::IfBlock(i) => on_if_block(i)
                .map(|i| Statement::IfBlock(i).at(pos))
                .unwrap(self),
            Statement::SelectCase(s) => self.convert(s).map(|s| Statement::SelectCase(s).at(pos)),
            Statement::ForLoop(f) => on_for_loop(f)
                .map(|f| Statement::ForLoop(f).at(pos))
                .unwrap(self),
            Statement::While(c) => on_conditional_block(c)
                .map(|c| Statement::While(c).at(pos))
                .unwrap(self),
            Statement::DoLoop(do_loop_node) => on_do_loop(do_loop_node)
                .map(|d| Statement::DoLoop(d).at(pos))
                .unwrap(self),
            Statement::Return(opt_label) => {
                if opt_label.is_some() && self.context.is_in_subprogram() {
                    // cannot have RETURN with explicit label inside subprogram
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
                } else {
                    Ok(Statement::Return(opt_label).at(pos))
                }
            }
            Statement::Resume(resume_option) => {
                if self.context.is_in_subprogram() {
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
                } else {
                    Ok(Statement::Resume(resume_option).at(pos))
                }
            }
            Statement::Exit(exit_object) => match self.context.get_name_context() {
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
            Statement::Dim(dim_list) => self
                .context
                .on_dim(dim_list, DimContext::Default)
                .map(|dim_list| Statement::Dim(dim_list).at(pos)),
            Statement::Redim(dim_list) => self
                .context
                .on_dim(dim_list, DimContext::Redim)
                .map(|dim_list| Statement::Redim(dim_list).at(pos)),
            Statement::Print(print_node) => self
                .convert(print_node)
                .map(|p| Statement::Print(p).at(pos)),
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

pub struct StatementsRemovingConstantsStateful {
    statements: StatementNodes,
}

impl StatementsRemovingConstantsStateful {
    pub fn new(statements: StatementNodes) -> Self {
        Self { statements }
    }
}

impl Stateful for StatementsRemovingConstantsStateful {
    type Output = StatementNodes;
    type State = ConverterImpl;
    type Error = QErrorNode;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        state.convert_block_removing_constants(self.statements)
    }
}
