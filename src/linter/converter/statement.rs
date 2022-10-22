use crate::common::*;
use crate::linter::converter::converter::{Context, Convertible};
use crate::linter::converter::ExprContext;
use crate::linter::{DimContext, NameContext};
use crate::parser::{ExitObject, Statement, StatementNode, StatementNodes};

impl Convertible<Context, Option<StatementNode>> for StatementNode {
    fn convert(self, ctx: &mut Context) -> Result<Option<StatementNode>, QErrorNode> {
        let Locatable {
            element: statement,
            pos,
        } = self;
        match statement {
            Statement::Assignment(n, e) => ctx.assignment(n.at(pos), e).map(Some),
            // CONST will be filtered out in the StatementNodes processor
            Statement::Const(n, e) => ctx.on_const(n, e).map(|_| None),
            Statement::SubCall(n, args) => ctx.sub_call(n.at(pos), args).map(Some),
            Statement::BuiltInSubCall(built_in_sub, args) => {
                let converted_args = ctx.on_expressions(args, ExprContext::Parameter)?;
                Ok(Statement::BuiltInSubCall(built_in_sub, converted_args).at(pos)).map(Some)
            }
            Statement::IfBlock(i) => i
                .convert(ctx)
                .map(|i| Statement::IfBlock(i).at(pos))
                .map(Some),
            Statement::SelectCase(s) => s
                .convert(ctx)
                .map(|s| Statement::SelectCase(s).at(pos))
                .map(Some),
            Statement::ForLoop(f) => f
                .convert(ctx)
                .map(|f| Statement::ForLoop(f).at(pos))
                .map(Some),
            Statement::While(c) => c
                .convert(ctx)
                .map(|c| Statement::While(c).at(pos))
                .map(Some),
            Statement::DoLoop(do_loop_node) => do_loop_node
                .convert(ctx)
                .map(|d| Statement::DoLoop(d).at(pos))
                .map(Some),
            Statement::Return(opt_label) => {
                if opt_label.is_some() && ctx.is_in_subprogram() {
                    // cannot have RETURN with explicit label inside subprogram
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
                } else {
                    Ok(Statement::Return(opt_label).at(pos)).map(Some)
                }
            }
            Statement::Resume(resume_option) => {
                if ctx.is_in_subprogram() {
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
                } else {
                    Ok(Statement::Resume(resume_option).at(pos)).map(Some)
                }
            }
            Statement::Exit(exit_object) => match ctx.get_name_context() {
                NameContext::Global => {
                    Err(QError::syntax_error("Illegal outside of subprogram")).with_err_at(pos)
                }
                NameContext::Sub => {
                    if exit_object == ExitObject::Sub {
                        Ok(Statement::Exit(exit_object).at(pos)).map(Some)
                    } else {
                        Err(QError::syntax_error("Illegal inside sub")).with_err_at(pos)
                    }
                }
                NameContext::Function => {
                    if exit_object == ExitObject::Function {
                        Ok(Statement::Exit(exit_object).at(pos)).map(Some)
                    } else {
                        Err(QError::syntax_error("Illegal inside function")).with_err_at(pos)
                    }
                }
            },
            Statement::Dim(dim_list) => ctx
                .on_dim(dim_list, DimContext::Default)
                .map(|dim_list| Statement::Dim(dim_list).at(pos))
                .map(Some),
            Statement::Redim(dim_list) => ctx
                .on_dim(dim_list, DimContext::Redim)
                .map(|dim_list| Statement::Redim(dim_list).at(pos))
                .map(Some),
            Statement::Print(print_node) => print_node
                .convert(ctx)
                .map(Statement::Print)
                .map(|s| s.at(pos))
                .map(Some),
            Statement::OnError(_)
            | Statement::Label(_)
            | Statement::GoTo(_)
            | Statement::GoSub(_)
            | Statement::Comment(_)
            | Statement::End
            | Statement::System => Ok(statement.at(pos)).map(Some),
        }
    }
}

impl Convertible for StatementNodes {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        self.into_iter()
            .map(|s| s.convert(ctx))
            .map(|res| match res {
                Ok(Some(s)) => Some(Ok(s)),
                Err(e) => Some(Err(e)),
                Ok(None) => None,
            })
            .flat_map(|opt| opt.into_iter())
            .collect()
    }
}
