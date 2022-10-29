mod assignment;
mod const_rules;
mod do_loop;
mod for_loop;
mod if_blocks;
mod print_node;
mod select_case;
mod sub_call;

use crate::linter::converter::context::Context;
use crate::linter::converter::pos_context::PosContext;
use crate::linter::converter::traits::Convertible;
use crate::linter::converter::types::{DimContext, ExprContext};
use crate::linter::NameContext;
use crate::parser::{ExitObject, Statement, StatementNode, StatementNodes};
use rusty_common::*;

impl Convertible<Context, Option<StatementNode>> for StatementNode {
    fn convert(self, ctx: &mut Context) -> Result<Option<StatementNode>, QErrorNode> {
        let Locatable {
            element: statement,
            pos,
        } = self;
        match statement.convert_in(ctx, pos) {
            Ok(Some(statement)) => Ok(Some(statement.at(pos))),
            Ok(None) => Ok(None),
            Err(err) => Err(err.patch_pos(pos)),
        }
    }
}

impl<'a> Convertible<PosContext<'a>, Option<Statement>> for Statement {
    fn convert(self, ctx: &mut PosContext) -> Result<Option<Statement>, QErrorNode> {
        match self {
            Statement::Assignment(n, e) => assignment::on_assignment(n, e, ctx).map(Some),
            // CONST will be filtered out in the StatementNodes processor
            Statement::Const(n, e) => const_rules::on_const(ctx, n, e).map(|_| None),
            Statement::SubCall(n, args) => ctx.sub_call(n, args).map(Some),
            Statement::BuiltInSubCall(built_in_sub, args) => {
                let converted_args = args.convert_in(ctx, ExprContext::Argument)?;
                Ok(Statement::BuiltInSubCall(built_in_sub, converted_args)).map(Some)
            }
            Statement::IfBlock(i) => i.convert(ctx).map(Statement::IfBlock).map(Some),
            Statement::SelectCase(s) => s.convert(ctx).map(Statement::SelectCase).map(Some),
            Statement::ForLoop(f) => f.convert(ctx).map(Statement::ForLoop).map(Some),
            Statement::While(c) => c.convert(ctx).map(Statement::While).map(Some),
            Statement::DoLoop(do_loop_node) => {
                do_loop_node.convert(ctx).map(Statement::DoLoop).map(Some)
            }
            Statement::Return(opt_label) => {
                if opt_label.is_some() && ctx.is_in_subprogram() {
                    // cannot have RETURN with explicit label inside subprogram
                    Err(QError::IllegalInSubFunction).with_err_no_pos()
                } else {
                    Ok(Statement::Return(opt_label)).map(Some)
                }
            }
            Statement::Resume(resume_option) => {
                if ctx.is_in_subprogram() {
                    Err(QError::IllegalInSubFunction).with_err_no_pos()
                } else {
                    Ok(Statement::Resume(resume_option)).map(Some)
                }
            }
            Statement::Exit(exit_object) => match ctx.names.get_name_context() {
                NameContext::Global => {
                    Err(QError::syntax_error("Illegal outside of subprogram")).with_err_no_pos()
                }
                NameContext::Sub => {
                    if exit_object == ExitObject::Sub {
                        Ok(Statement::Exit(exit_object)).map(Some)
                    } else {
                        Err(QError::syntax_error("Illegal inside sub")).with_err_no_pos()
                    }
                }
                NameContext::Function => {
                    if exit_object == ExitObject::Function {
                        Ok(Statement::Exit(exit_object)).map(Some)
                    } else {
                        Err(QError::syntax_error("Illegal inside function")).with_err_no_pos()
                    }
                }
            },
            Statement::Dim(dim_list) => dim_list
                .convert_in_default(ctx)
                .map(Statement::Dim)
                .map(Some),
            Statement::Redim(dim_list) => dim_list
                .convert_in(ctx, DimContext::Redim)
                .map(Statement::Redim)
                .map(Some),
            Statement::Print(print_node) => print_node.convert(ctx).map(Statement::Print).map(Some),
            Statement::OnError(_)
            | Statement::Label(_)
            | Statement::GoTo(_)
            | Statement::GoSub(_)
            | Statement::Comment(_)
            | Statement::End
            | Statement::System => Ok(self).map(Some),
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
