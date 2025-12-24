use crate::converter::common::Context;
use crate::converter::common::Convertible;
use crate::converter::common::PosContext;
use crate::converter::common::{DimContext, ExprContext};
use crate::converter::statement::{assignment, const_rules};
use crate::core::{LintError, LintErrorPos};
use crate::core::{LintPosResult, NameContext};
use rusty_common::*;
use rusty_parser::{ExitObject, Statement, StatementPos, Statements};

impl Convertible<Context, Option<Self>> for StatementPos {
    fn convert(self, ctx: &mut Context) -> Result<Option<Self>, LintErrorPos> {
        let Self {
            element: statement,
            pos,
        } = self;
        statement
            .convert_in(ctx, pos)
            .map(|opt_statement| opt_statement.map(|s| s.at_pos(pos)))
            .patch_err_pos(&pos)
    }
}

impl<'a> Convertible<PosContext<'a>, Option<Self>> for Statement {
    fn convert(self, ctx: &mut PosContext) -> Result<Option<Self>, LintErrorPos> {
        match self {
            Self::Assignment(n, e) => assignment::on_assignment(n, e, ctx).map(Some),
            // CONST is mapped to None and is filtered out
            Self::Const(n, e) => const_rules::on_const(ctx, n, e).map(|_| None),
            Self::SubCall(n, args) => ctx.sub_call(n, args).map(Some),
            Self::BuiltInSubCall(built_in_sub, args) => {
                let converted_args = args.convert_in(ctx, ExprContext::Argument)?;
                Ok(Self::BuiltInSubCall(built_in_sub, converted_args)).map(Some)
            }
            Self::IfBlock(i) => i.convert(ctx).map(Statement::IfBlock).map(Some),
            Self::SelectCase(s) => s.convert(ctx).map(Statement::SelectCase).map(Some),
            Self::ForLoop(f) => f.convert(ctx).map(Statement::ForLoop).map(Some),
            Self::While(c) => c.convert(ctx).map(Statement::While).map(Some),
            Self::DoLoop(do_loop) => do_loop.convert(ctx).map(Statement::DoLoop).map(Some),
            Self::Return(opt_label) => {
                if opt_label.is_some() && ctx.is_in_subprogram() {
                    // cannot have RETURN with explicit label inside subprogram
                    Err(LintError::IllegalInSubFunction.at_no_pos())
                } else {
                    Ok(Self::Return(opt_label)).map(Some)
                }
            }
            Self::Resume(resume_option) => {
                if ctx.is_in_subprogram() {
                    Err(LintError::IllegalInSubFunction.at_no_pos())
                } else {
                    Ok(Self::Resume(resume_option)).map(Some)
                }
            }
            Self::Exit(exit_object) => match ctx.names.get_name_context() {
                NameContext::Global => Err(LintError::IllegalOutsideSubFunction.at_no_pos()),
                NameContext::Sub => {
                    if exit_object == ExitObject::Sub {
                        Ok(Self::Exit(exit_object)).map(Some)
                    } else {
                        Err(LintError::IllegalInSubFunction.at_no_pos())
                    }
                }
                NameContext::Function => {
                    if exit_object == ExitObject::Function {
                        Ok(Self::Exit(exit_object)).map(Some)
                    } else {
                        Err(LintError::IllegalInSubFunction.at_no_pos())
                    }
                }
            },
            Self::Dim(dim_list) => dim_list
                .convert_in_default(ctx)
                .map(Statement::Dim)
                .map(Some),
            Self::Redim(dim_list) => dim_list
                .convert_in(ctx, DimContext::Redim)
                .map(Statement::Redim)
                .map(Some),
            Self::Print(print) => print.convert(ctx).map(Statement::Print).map(Some),
            Self::OnError(_)
            | Self::Label(_)
            | Self::GoTo(_)
            | Self::GoSub(_)
            | Self::Comment(_)
            | Self::End
            | Self::System => Ok(self).map(Some),
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
