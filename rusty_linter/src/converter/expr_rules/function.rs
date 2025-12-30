use rusty_common::{AtPos, Position};
use rusty_parser::{
    AsBareName, BareName, Expression, ExpressionType, Expressions, Name, VariableInfo
};

use crate::converter::common::{Context, ConvertibleIn, ExprContext, ExprContextPos};
use crate::converter::expr_rules::qualify_name::*;
use crate::core::{IntoQualified, IntoTypeQualifier, LintError, LintErrorPos, LintResult};

pub fn convert(
    ctx: &mut Context,
    extra: ExprContextPos,
    name: Name,
    args: Expressions,
) -> Result<Expression, LintErrorPos> {
    // ExistingArrayWithParenthesis goes first because they're allowed to have no arguments
    let rules: Vec<Box<dyn FuncResolve>> = vec![Box::new(ExistingArrayWithParenthesis::default())];
    for mut rule in rules {
        if rule.can_handle(ctx, &name) {
            return rule.resolve(ctx, extra, name, args);
        }
    }

    // now validate we have arguments
    functions_must_have_arguments(&args, extra.pos)?;
    // continue with built-in/user defined functions
    resolve_function(ctx, name, args, extra.pos)
}

fn resolve_function(
    ctx: &mut Context,
    name: Name,
    args: Expressions,
    pos: Position,
) -> Result<Expression, LintErrorPos> {
    // convert args
    let converted_args = convert_function_args(ctx, args)?;
    // is it built-in function?
    let converted_expr = match try_built_in_function(&name).with_err_at(&pos)? {
        Some(built_in_function) => {
            Expression::BuiltInFunctionCall(built_in_function, converted_args)
        }
        _ => {
            let converted_name: Name = match ctx.function_qualifier(name.as_bare_name()) {
                Some(function_qualifier) => {
                    try_qualify(name, function_qualifier).with_err_at(&pos)?
                }
                _ => name.to_qualified(ctx),
            };
            Expression::FunctionCall(converted_name, converted_args)
        }
    };
    Ok(converted_expr)
}

trait FuncResolve {
    fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool;

    fn resolve(
        &self,
        ctx: &mut Context,
        extra: ExprContextPos,
        name: Name,
        args: Expressions,
    ) -> Result<Expression, LintErrorPos>;
}

#[derive(Default)]
struct ExistingArrayWithParenthesis {
    var_info: Option<VariableInfo>,
}

impl ExistingArrayWithParenthesis {
    fn is_array(&self) -> bool {
        match &self.var_info {
            Some(var_info) => matches!(&var_info.expression_type, ExpressionType::Array(_)),
            _ => false,
        }
    }

    fn get_var_info<'a>(ctx: &'a Context, name: &Name) -> Option<&'a VariableInfo> {
        Self::get_extended_var_info(ctx, name.as_bare_name())
            .or_else(|| Self::get_compact_var_info(ctx, name))
    }

    fn get_extended_var_info<'a>(
        ctx: &'a Context,
        bare_name: &BareName,
    ) -> Option<&'a VariableInfo> {
        ctx.names.get_extended_var_recursively(bare_name)
    }

    fn get_compact_var_info<'a>(ctx: &'a Context, name: &Name) -> Option<&'a VariableInfo> {
        let qualifier = name.qualify(ctx);
        ctx.names
            .get_compact_var_recursively(name.as_bare_name(), qualifier)
    }
}

impl FuncResolve for ExistingArrayWithParenthesis {
    fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool {
        self.var_info = Self::get_var_info(ctx, name).map(Clone::clone);
        self.is_array()
    }

    fn resolve(
        &self,
        ctx: &mut Context,
        extra: ExprContextPos,
        name: Name,
        args: Expressions,
    ) -> Result<Expression, LintErrorPos> {
        // convert args
        let converted_args = args.convert_in(ctx, extra.element)?;
        // convert name
        let VariableInfo {
            expression_type,
            shared,
            redim_info,
        } = self.var_info.clone().unwrap();
        match expression_type {
            ExpressionType::Array(element_type) => {
                let converted_name =
                    qualify_name(element_type.as_ref(), name).with_err_at(&extra.pos)?;
                // create result
                let result_expr = Expression::ArrayElement(
                    converted_name,
                    converted_args,
                    VariableInfo {
                        expression_type: element_type.as_ref().clone(),
                        shared,
                        redim_info,
                    },
                );
                Ok(result_expr)
            }
            _ => Err(LintError::ArrayNotDefined.at_pos(extra.pos)),
        }
    }
}

pub fn functions_must_have_arguments(
    args: &Expressions,
    pos: Position,
) -> Result<(), LintErrorPos> {
    if args.is_empty() {
        Err(LintError::FunctionNeedsArguments.at_pos(pos))
    } else {
        Ok(())
    }
}

pub fn convert_function_args(
    ctx: &mut Context,
    args: Expressions,
) -> Result<Expressions, LintErrorPos> {
    args.convert_in(ctx, ExprContext::Argument)
}
