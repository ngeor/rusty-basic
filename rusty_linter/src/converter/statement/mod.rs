mod assignment;
mod const_rules;
mod do_loop;
mod for_loop;
mod if_blocks;
mod print;
mod select_case;
mod sub_call;

use crate::converter::context::Context;
use crate::converter::pos_context::PosContext;
use crate::converter::traits::Convertible;
use crate::converter::types::{DimContext, ExprContext};
use crate::error::{LintError, LintErrorPos};
use crate::NameContext;
use rusty_common::*;
use rusty_parser::{ExitObject, Statement, StatementPos, Statements};

impl Convertible<Context, Option<StatementPos>> for StatementPos {
    fn convert(self, ctx: &mut Context) -> Result<Option<StatementPos>, LintErrorPos> {
        let Positioned {
            element: statement,
            pos,
        } = self;
        match statement.convert_in(ctx, pos) {
            Ok(Some(statement)) => Ok(Some(statement.at_pos(pos))),
            Ok(None) => Ok(None),
            Err(err) => Err(err.patch_pos(pos)),
        }
    }
}

impl<'a> Convertible<PosContext<'a>, Option<Statement>> for Statement {
    fn convert(self, ctx: &mut PosContext) -> Result<Option<Statement>, LintErrorPos> {
        match self {
            Statement::Assignment(n, e) => assignment::on_assignment(n, e, ctx).map(Some),
            // CONST is mapped to None and is filtered out
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
            Statement::DoLoop(do_loop) => do_loop.convert(ctx).map(Statement::DoLoop).map(Some),
            Statement::Return(opt_label) => {
                if opt_label.is_some() && ctx.is_in_subprogram() {
                    // cannot have RETURN with explicit label inside subprogram
                    Err(LintError::IllegalInSubFunction).with_err_no_pos()
                } else {
                    Ok(Statement::Return(opt_label)).map(Some)
                }
            }
            Statement::Resume(resume_option) => {
                if ctx.is_in_subprogram() {
                    Err(LintError::IllegalInSubFunction).with_err_no_pos()
                } else {
                    Ok(Statement::Resume(resume_option)).map(Some)
                }
            }
            Statement::Exit(exit_object) => match ctx.names.get_name_context() {
                NameContext::Global => Err(LintError::IllegalOutsideSubFunction).with_err_no_pos(),
                NameContext::Sub => {
                    if exit_object == ExitObject::Sub {
                        Ok(Statement::Exit(exit_object)).map(Some)
                    } else {
                        Err(LintError::IllegalInSubFunction).with_err_no_pos()
                    }
                }
                NameContext::Function => {
                    if exit_object == ExitObject::Function {
                        Ok(Statement::Exit(exit_object)).map(Some)
                    } else {
                        Err(LintError::IllegalInSubFunction).with_err_no_pos()
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
            Statement::Print(print) => print.convert(ctx).map(Statement::Print).map(Some),
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

impl Convertible for Statements {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
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
