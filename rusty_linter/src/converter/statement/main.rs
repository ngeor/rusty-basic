use crate::converter::common::Context;
use crate::converter::common::Convertible;
use crate::converter::common::ConvertibleIn;
use crate::converter::common::{DimContext, ExprContext};
use crate::converter::statement::{assignment, const_rules};
use crate::core::NameContext;
use crate::core::{LintError, LintErrorPos};
use rusty_common::*;
use rusty_parser::{ExitObject, Statement};

impl ConvertibleIn<Position> for Statement {
    fn convert_in(self, ctx: &mut Context, pos: Position) -> Result<Self, LintErrorPos> {
        match self {
            Self::Assignment(n, e) => assignment::on_assignment(n, e, ctx, pos),
            // CONST is mapped to None and is filtered out
            Self::Const(n, e) => const_rules::on_const(ctx, n, e),
            Self::SubCall(n, args) => ctx.sub_call(n, args),
            Self::BuiltInSubCall(built_in_sub, args) => {
                let converted_args = args.convert_in(ctx, ExprContext::Argument)?;
                Ok(Self::BuiltInSubCall(built_in_sub, converted_args))
            }
            Self::IfBlock(i) => i.convert(ctx).map(Statement::IfBlock),
            Self::SelectCase(s) => s.convert(ctx).map(Statement::SelectCase),
            Self::ForLoop(f) => f.convert(ctx).map(Statement::ForLoop),
            Self::While(c) => c.convert(ctx).map(Statement::While),
            Self::DoLoop(do_loop) => do_loop.convert(ctx).map(Statement::DoLoop),
            Self::Return(opt_label) => {
                if opt_label.is_some() && ctx.is_in_subprogram() {
                    // cannot have RETURN with explicit label inside subprogram
                    Err(LintError::IllegalInSubFunction.at_pos(pos))
                } else {
                    Ok(Self::Return(opt_label))
                }
            }
            Self::Resume(resume_option) => {
                if ctx.is_in_subprogram() {
                    Err(LintError::IllegalInSubFunction.at_pos(pos))
                } else {
                    Ok(Self::Resume(resume_option))
                }
            }
            Self::Exit(exit_object) => match ctx.names.get_name_context() {
                NameContext::Global => Err(LintError::IllegalOutsideSubFunction.at_pos(pos)),
                NameContext::Sub => {
                    if exit_object == ExitObject::Sub {
                        Ok(Self::Exit(exit_object))
                    } else {
                        Err(LintError::IllegalInSubFunction.at_pos(pos))
                    }
                }
                NameContext::Function => {
                    if exit_object == ExitObject::Function {
                        Ok(Self::Exit(exit_object))
                    } else {
                        Err(LintError::IllegalInSubFunction.at_pos(pos))
                    }
                }
            },
            Self::Dim(dim_list) => dim_list.convert_in_default(ctx).map(Statement::Dim),
            Self::Redim(dim_list) => dim_list
                .convert_in(ctx, DimContext::Redim)
                .map(Statement::Redim),
            Self::Print(print) => print.convert(ctx).map(Statement::Print),
            Self::OnError(_)
            | Self::Label(_)
            | Self::GoTo(_)
            | Self::GoSub(_)
            | Self::Comment(_)
            | Self::End
            | Self::System => Ok(self),
        }
    }
}
