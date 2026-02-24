use rusty_common::{AtPos, Position};
use rusty_parser::{
    AsBareName, BuiltInFunction, BuiltInStyle, DimType, Expression, ExpressionType, Name, TypeQualifier, VariableInfo
};
use rusty_variant::Variant;

use crate::converter::common::{ExprContext, ExprContextPos};
use crate::converter::expr_rules::qualify_name::*;
use crate::core::{
    ConstLookup, Context, IntoQualified, IntoTypeQualifier, LintError, LintErrorPos, LintResult, qualifier_of_const_variant
};

pub fn convert(
    ctx: &mut Context,
    extra: ExprContextPos,
    name: Name,
    variable_info: VariableInfo,
) -> Result<Expression, LintErrorPos> {
    // validation rules
    validate(ctx, &name, extra.pos)?;

    // match existing
    let mut rules: Vec<Box<dyn VarResolve>> = vec![];
    rules.push(Box::new(ExistingVar::default()));
    rules.push(Box::new(ExistingConst::new_local()));
    if extra.element != ExprContext::Default {
        rules.push(Box::new(AssignToFunction::default()));
    } else {
        rules.push(Box::new(VarAsBuiltInFunctionCall::default()));
        rules.push(Box::new(VarAsUserDefinedFunctionCall::default()));
    }
    rules.push(Box::new(ExistingConst::new_recursive()));

    for mut rule in rules {
        if rule.can_handle(ctx, &name) {
            return rule.resolve(ctx, extra.clone(), name);
        }
    }

    if extra.element != ExprContext::ResolvingPropertyOwner {
        // add as new implicit
        Ok(add_as_new_implicit_var(ctx, extra, name))
    } else {
        // repack as unresolved
        Ok(Expression::Variable(name, variable_info))
    }
}

fn validate(ctx: &Context, name: &Name, pos: Position) -> Result<(), LintErrorPos> {
    if ctx.subs.contains_key(name.as_bare_name()) {
        return Err(LintError::DuplicateDefinition.at_pos(pos));
    }

    Ok(())
}

pub fn add_as_new_implicit_var(ctx: &mut Context, extra: ExprContextPos, name: Name) -> Expression {
    let resolved_name = name.to_qualified(ctx);

    let bare_name = resolved_name.as_bare_name();
    let q = resolved_name.qualifier().expect("Should be resolved");
    ctx.names.insert(
        bare_name.clone(),
        &DimType::BuiltIn(q, BuiltInStyle::Compact),
        false,
        None,
    );

    let var_info = VariableInfo::new_built_in(q, false);
    let pos = extra.pos;
    ctx.names
        .get_implicit_vars_mut()
        .push(resolved_name.clone().demand_qualified().at_pos(pos));
    Expression::Variable(resolved_name, var_info)
}

pub trait VarResolve {
    fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool;

    fn resolve(
        &self,
        ctx: &mut Context,
        extra: ExprContextPos,
        name: Name,
    ) -> Result<Expression, LintErrorPos>;
}

#[derive(Default)]
pub struct ExistingVar {
    var_info: Option<VariableInfo>,
}

impl VarResolve for ExistingVar {
    fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool {
        let bare_name = name.as_bare_name();
        self.var_info = ctx.names.get_extended_var_recursively(bare_name).cloned();
        if self.var_info.is_some() {
            true
        } else {
            let q = name.qualify(ctx);
            self.var_info = ctx.names.get_compact_var_recursively(bare_name, q).cloned();
            self.var_info.is_some()
        }
    }

    fn resolve(
        &self,
        _ctx: &mut Context,
        extra: ExprContextPos,
        name: Name,
    ) -> Result<Expression, LintErrorPos> {
        let variable_info = self.var_info.clone().unwrap();
        let expression_type = &variable_info.expression_type;
        let converted_name = qualify_name(expression_type, name).with_err_at(&extra.pos)?;
        Ok(Expression::Variable(converted_name, variable_info))
    }
}

pub struct ExistingConst {
    use_recursion: bool,
    opt_v: Option<Variant>,
}

impl ExistingConst {
    pub fn new_local() -> Self {
        Self {
            use_recursion: false,
            opt_v: None,
        }
    }

    pub fn new_recursive() -> Self {
        Self {
            use_recursion: true,
            opt_v: None,
        }
    }
}

impl VarResolve for ExistingConst {
    fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool {
        self.opt_v = if self.use_recursion {
            ctx.names
                .get_const_value_recursively(name.as_bare_name())
                .cloned()
        } else {
            ctx.names
                .names()
                .get_const_value(name.as_bare_name())
                .cloned()
        };
        self.opt_v.is_some()
    }

    fn resolve(
        &self,
        _ctx: &mut Context,
        extra: ExprContextPos,
        name: Name,
    ) -> Result<Expression, LintErrorPos> {
        let v = self.opt_v.clone().unwrap();
        let q = qualifier_of_const_variant(&v);
        if name.is_bare_or_of_type(q) {
            // resolve to literal expression
            Ok(const_variant_to_expression(v))
        } else {
            Err(LintError::DuplicateDefinition.at_pos(extra.pos))
        }
    }
}

fn const_variant_to_expression(value: Variant) -> Expression {
    match value {
        Variant::VSingle(f) => f.into(),
        Variant::VDouble(d) => d.into(),
        Variant::VString(s) => s.into(),
        Variant::VInteger(i) => i.into(),
        Variant::VLong(l) => l.into(),
        _ => panic!("Invalid const variant: {:?}", value),
    }
}

#[derive(Default)]
pub struct AssignToFunction {
    function_qualifier: Option<TypeQualifier>,
}

impl VarResolve for AssignToFunction {
    fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool {
        let bare_name = name.as_bare_name();
        match ctx.function_qualifier(bare_name) {
            Some(function_qualifier) => {
                self.function_qualifier = Some(function_qualifier);
                true
            }
            _ => false,
        }
    }

    fn resolve(
        &self,
        ctx: &mut Context,
        extra: ExprContextPos,
        name: Name,
    ) -> Result<Expression, LintErrorPos> {
        let function_qualifier = self.function_qualifier.unwrap();
        if ctx.names.is_in_function(name.as_bare_name()) {
            let converted_name = try_qualify(name, function_qualifier).with_err_at(&extra.pos)?;
            let expr_type = ExpressionType::BuiltIn(function_qualifier);
            let variable_info = VariableInfo::new_local(expr_type);

            // store this in the name context too to make it easier for instruction generator
            // TODO add unit test
            // TODO what if the name already exists?
            ctx.names
                .insert_compact(converted_name.as_bare_name().clone(), variable_info.clone());

            let expr = Expression::Variable(converted_name, variable_info);

            Ok(expr)
        } else {
            Err(LintError::DuplicateDefinition.at_pos(extra.pos))
        }
    }
}

#[derive(Default)]
pub struct VarAsBuiltInFunctionCall {
    built_in_function: Option<BuiltInFunction>,
}

impl VarResolve for VarAsBuiltInFunctionCall {
    fn can_handle(&mut self, _ctx: &Context, name: &Name) -> bool {
        self.built_in_function = BuiltInFunction::try_parse(name.as_bare_name());
        self.built_in_function.is_some()
    }

    fn resolve(
        &self,
        _ctx: &mut Context,
        extra: ExprContextPos,
        name: Name,
    ) -> Result<Expression, LintErrorPos> {
        match try_built_in_function(&name).with_err_at(&extra.pos)? {
            Some(built_in_function) => {
                Ok(Expression::BuiltInFunctionCall(built_in_function, vec![]))
            }
            _ => panic!("VarAsBuiltInFunctionCall::resolve should not have been called"),
        }
    }
}

#[derive(Default)]
pub struct VarAsUserDefinedFunctionCall {
    function_qualifier: Option<TypeQualifier>,
}

impl VarResolve for VarAsUserDefinedFunctionCall {
    fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool {
        self.function_qualifier = ctx.function_qualifier(name.as_bare_name());
        self.function_qualifier.is_some()
    }

    fn resolve(
        &self,
        _ctx: &mut Context,
        extra: ExprContextPos,
        name: Name,
    ) -> Result<Expression, LintErrorPos> {
        let q = self.function_qualifier.unwrap();
        let converted_name = try_qualify(name, q).with_err_at(&extra.pos)?;
        Ok(Expression::FunctionCall(converted_name, vec![]))
    }
}
