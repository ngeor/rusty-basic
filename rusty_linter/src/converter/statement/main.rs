use rusty_common::*;
use rusty_parser::{ExitObject, Statement};

use crate::converter::common::{Convertible, ConvertibleIn, DimContext, ExprContext};
use crate::converter::statement::{assignment, const_rules};
use crate::core::{Context, LintError, LintErrorPos, ScopeKind};

impl ConvertibleIn<Position> for Statement {
    fn convert_in(self, ctx: &mut Context, pos: Position) -> Result<Self, LintErrorPos> {
        match self {
            Self::Assignment(a) => assignment::on_assignment(a, ctx, pos),
            // CONST is mapped to None and is filtered out
            Self::Const(c) => const_rules::on_const(ctx, c),
            Self::SubCall(sub_call) => ctx.sub_call(sub_call),
            Self::BuiltInSubCall(sub_call) => {
                let (built_in_sub, args) = sub_call.into();
                let converted_args = args.convert_in(ctx, ExprContext::Argument)?;
                Ok(Self::built_in_sub_call(built_in_sub, converted_args))
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
            Self::Exit(exit_object) => match ctx.names.get_current_scope_kind() {
                ScopeKind::Global => Err(LintError::IllegalOutsideSubFunction.at_pos(pos)),
                ScopeKind::Sub => {
                    if exit_object == ExitObject::Sub {
                        Ok(Self::Exit(exit_object))
                    } else {
                        Err(LintError::IllegalInSubFunction.at_pos(pos))
                    }
                }
                ScopeKind::Function => {
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
