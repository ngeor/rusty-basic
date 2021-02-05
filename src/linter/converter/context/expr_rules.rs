use std::convert::TryFrom;

use super::{Context, ExprContext};
use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::linter::type_resolver::TypeResolver;
use crate::parser::*;
use crate::variant::Variant;

type O = (ExpressionNode, Vec<QualifiedNameNode>);
type R = Result<O, QErrorNode>;

pub fn on_expression(
    ctx: &mut Context,
    expr_node: ExpressionNode,
    expr_context: ExprContext,
) -> Result<O, QErrorNode> {
    let Locatable { element: expr, pos } = expr_node;
    match expr {
        // literals
        Expression::SingleLiteral(f) => ok_no_implicits(Expression::SingleLiteral(f), pos),
        Expression::DoubleLiteral(d) => ok_no_implicits(Expression::DoubleLiteral(d), pos),
        Expression::StringLiteral(s) => ok_no_implicits(Expression::StringLiteral(s), pos),
        Expression::IntegerLiteral(i) => ok_no_implicits(Expression::IntegerLiteral(i), pos),
        Expression::LongLiteral(l) => ok_no_implicits(Expression::LongLiteral(l), pos),
        // parenthesis
        Expression::Parenthesis(box_child) => {
            parenthesis_v2::convert(ctx, box_child, expr_context, pos)
        }
        // unary
        Expression::UnaryExpression(unary_operator, box_child) => {
            unary_v2::convert(ctx, unary_operator, box_child, expr_context, pos)
        }
        // binary
        Expression::BinaryExpression(binary_operator, left, right, _expr_type) => {
            binary_v2::convert(ctx, binary_operator, left, right, expr_context, pos)
        }
        // variables
        Expression::Variable(name, variable_info) => {
            variable_v2::convert(ctx, name, variable_info, expr_context, pos)
        }
        Expression::ArrayElement(_name, _indices, _variable_info) => {
            panic!(
                "Parser is not supposed to produce any ArrayElement expressions, only FunctionCall"
            )
        }
        Expression::Property(box_left_side, property_name, _expr_type) => {
            property_v2::convert(ctx, box_left_side, property_name, expr_context, pos)
        }
        // function call
        Expression::FunctionCall(name, args) => {
            function_v2::convert(ctx, name, args, expr_context, pos)
        }
        Expression::BuiltInFunctionCall(_built_in_function, _args) => {
            panic!("Parser is not supposed to produce any BuiltInFunctionCall expressions, only FunctionCall")
        }
    }
}

fn ok_no_implicits(expr: Expression, pos: Location) -> R {
    Ok((expr.at(pos), vec![]))
}

pub mod parenthesis_v2 {
    use super::*;

    pub fn convert(
        ctx: &mut Context,
        box_child: Box<ExpressionNode>,
        expr_context: ExprContext,
        pos: Location,
    ) -> R {
        // convert child (recursion)
        let (converted_child, implicit_vars) = ctx.on_expression(*box_child, expr_context)?;
        let parenthesis_expr = Expression::Parenthesis(Box::new(converted_child));
        Ok((parenthesis_expr.at(pos), implicit_vars))
    }
}

pub mod unary_v2 {
    use super::*;
    pub fn convert(
        ctx: &mut Context,
        unary_operator: UnaryOperator,
        box_child: Box<ExpressionNode>,
        expr_context: ExprContext,
        pos: Location,
    ) -> R {
        // convert child (recursion)
        let (converted_child, implicit_vars) = ctx.on_expression(*box_child, expr_context)?;
        // ensure operator applies to converted expr
        let converted_expr_type = converted_child.expression_type();
        if unary_operator.applies_to(&converted_expr_type) {
            let unary_expr = Expression::UnaryExpression(unary_operator, Box::new(converted_child));
            Ok((unary_expr.at(pos), implicit_vars))
        } else {
            Err(QError::TypeMismatch).with_err_at(&converted_child)
        }
    }
}

pub mod binary_v2 {
    use super::*;
    pub fn convert(
        ctx: &mut Context,
        binary_operator: Operator,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
        expr_context: ExprContext,
        pos: Location,
    ) -> R {
        let (converted_left, mut left_implicit_vars) = ctx.on_expression(*left, expr_context)?;
        let (converted_right, mut right_implicit_vars) = ctx.on_expression(*right, expr_context)?;
        let new_expr = Expression::binary(converted_left, converted_right, binary_operator)?;
        left_implicit_vars.append(&mut right_implicit_vars);
        Ok((new_expr.at(pos), left_implicit_vars))
    }
}

pub mod variable_v2 {
    use super::*;

    pub fn convert(
        ctx: &mut Context,
        name: Name,
        variable_info: VariableInfo,
        expr_context: ExprContext,
        pos: Location,
    ) -> R {
        // validation rules
        validate(ctx, &name, pos)?;

        // match existing
        let mut rules: Vec<Box<dyn VarResolve<'_>>> = vec![];
        rules.push(Box::new(ExistingVar::default()));
        rules.push(Box::new(ExistingConst::new_local()));
        if expr_context != ExprContext::Default {
            rules.push(Box::new(AssignToFunction::default()));
        } else {
            rules.push(Box::new(VarAsBuiltInFunctionCall::default()));
            rules.push(Box::new(VarAsUserDefinedFunctionCall::default()));
        }
        rules.push(Box::new(ExistingConst::new_recursive()));

        for mut rule in rules {
            if rule.can_handle(ctx, &name) {
                return rule.resolve_no_implicits(ctx, name, pos);
            }
        }

        if expr_context != ExprContext::ResolvingPropertyOwner {
            // add as new implicit
            Ok(add_as_new_implicit_var(ctx, name, pos))
        } else {
            // repack as unresolved
            Ok((Expression::Variable(name, variable_info).at(pos), vec![]))
        }
    }

    fn validate(ctx: &Context, name: &Name, pos: Location) -> Result<(), QErrorNode> {
        if ctx.subs.contains_key(name.bare_name()) {
            return Err(QError::DuplicateDefinition).with_err_at(pos);
        }

        Ok(())
    }

    pub fn add_as_new_implicit_var(ctx: &mut Context, name: Name, pos: Location) -> O {
        let resolved_name = ctx.resolve_name_to_name(name);
        let q_name = resolved_name.clone().demand_qualified();
        let qualifier = q_name.qualifier;
        let implicit_vars = vec![q_name.at(pos)];
        let expr_type = ExpressionType::BuiltIn(qualifier);
        let var_info = VariableInfo::new_local(expr_type);
        ctx.names.insert_compact(
            resolved_name.bare_name().clone(),
            qualifier,
            var_info.clone(),
        );
        let resolved_expr = Expression::Variable(resolved_name, var_info).at(pos);
        (resolved_expr, implicit_vars)
    }

    pub trait VarResolve<'a> {
        fn can_handle(&mut self, ctx: &'a Context, name: &Name) -> bool;

        fn resolve(
            &self,
            ctx: &'a Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode>;

        fn resolve_no_implicits(&self, ctx: &'a Context, name: Name, pos: Location) -> R {
            self.resolve(ctx, name, pos).map(|e| (e, vec![]))
        }
    }

    #[derive(Default)]
    pub struct ExistingVar<'a> {
        var_info: Option<&'a VariableInfo>,
    }

    impl<'a> VarResolve<'a> for ExistingVar<'a> {
        fn can_handle(&mut self, ctx: &'a Context, name: &Name) -> bool {
            let bare_name = name.bare_name();
            self.var_info = ctx.names.get_extended_var_recursively(bare_name);
            if self.var_info.is_some() {
                true
            } else {
                let q: TypeQualifier = ctx.resolve_name_to_qualifier(&name);
                self.var_info = ctx.names.get_compact_var_recursively(bare_name, q);
                self.var_info.is_some()
            }
        }

        fn resolve(
            &self,
            _ctx: &'a Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            let variable_info = self.var_info.unwrap();
            let expression_type = &variable_info.expression_type;
            let converted_name = expression_type
                .qualify_name(name.clone())
                .with_err_at(pos)?;
            Ok(Expression::Variable(converted_name, variable_info.clone()).at(pos))
        }
    }

    pub struct ExistingConst<'a> {
        use_recursion: bool,
        opt_v: Option<&'a Variant>,
    }

    impl<'a> ExistingConst<'a> {
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

    impl<'a> VarResolve<'a> for ExistingConst<'a> {
        fn can_handle(&mut self, ctx: &'a Context, name: &Name) -> bool {
            self.opt_v = if self.use_recursion {
                ctx.names.get_const_value_recursively(name.bare_name())
            } else {
                ctx.names.get_const_value_no_recursion(name.bare_name())
            };
            self.opt_v.is_some()
        }

        fn resolve(
            &self,
            _ctx: &'a Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            let v = self.opt_v.unwrap();
            let q = TypeQualifier::try_from(v).with_err_at(pos)?;
            if name.is_bare_or_of_type(q) {
                // resolve to literal expression
                let expr = Expression::try_from(v.clone()).with_err_at(pos)?;
                Ok(expr.at(pos))
            } else {
                Err(QError::DuplicateDefinition).with_err_at(pos)
            }
        }
    }

    #[derive(Default)]
    pub struct AssignToFunction {
        function_qualifier: Option<TypeQualifier>,
    }

    impl<'a> VarResolve<'a> for AssignToFunction {
        fn can_handle(&mut self, ctx: &'a Context, name: &Name) -> bool {
            let bare_name = name.bare_name();
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
            ctx: &'a Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            let function_qualifier = self.function_qualifier.unwrap();
            let bare_name = name.bare_name();
            if name.is_bare_or_of_type(function_qualifier) {
                if ctx.names.is_in_function(bare_name) {
                    let converted_name = name.qualify(function_qualifier);
                    let expr_type = ExpressionType::BuiltIn(function_qualifier);
                    let expr =
                        Expression::Variable(converted_name, VariableInfo::new_local(expr_type));
                    return Ok(expr.at(pos));
                }
            }

            Err(QError::DuplicateDefinition).with_err_at(pos)
        }
    }

    #[derive(Default)]
    pub struct VarAsBuiltInFunctionCall {
        built_in_function: Option<BuiltInFunction>,
    }

    impl<'a> VarResolve<'a> for VarAsBuiltInFunctionCall {
        fn can_handle(&mut self, _ctx: &'a Context, name: &Name) -> bool {
            self.built_in_function = name.bare_name().into();
            self.built_in_function.is_some()
        }

        fn resolve(
            &self,
            _ctx: &'a Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            match Option::<BuiltInFunction>::try_from(&name).with_err_at(pos)? {
                Some(built_in_function) => {
                    Ok(Expression::BuiltInFunctionCall(built_in_function, vec![]).at(pos))
                }
                _ => panic!("VarAsBuiltInFunctionCall::resolve should not have been called"),
            }
        }
    }

    #[derive(Default)]
    pub struct VarAsUserDefinedFunctionCall {
        function_qualifier: Option<TypeQualifier>,
    }

    impl<'a> VarResolve<'a> for VarAsUserDefinedFunctionCall {
        fn can_handle(&mut self, ctx: &'a Context<'a>, name: &Name) -> bool {
            self.function_qualifier = ctx.function_qualifier(name.bare_name());
            self.function_qualifier.is_some()
        }

        fn resolve(
            &self,
            _ctx: &'a Context<'a>,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            let converted_name = name.qualify(self.function_qualifier.unwrap());
            Ok(Expression::FunctionCall(converted_name, vec![]).at(pos))
        }
    }
}

pub mod property_v2 {
    use super::*;
    use crate::linter::converter::context::expr_rules::variable_v2::{
        add_as_new_implicit_var, AssignToFunction, ExistingConst, ExistingVar,
        VarAsUserDefinedFunctionCall, VarResolve,
    };

    pub fn convert(
        ctx: &mut Context,
        left_side: Box<Expression>,
        property_name: Name,
        expr_context: ExprContext,
        pos: Location,
    ) -> R {
        // can we fold it into a name?
        let opt_folded_name = try_fold(&left_side, property_name.clone());
        if let Some(folded_name) = opt_folded_name {
            // checking out if we have an existing variable / const etc that contains a dot
            let mut rules: Vec<Box<dyn VarResolve<'_>>> = vec![];
            rules.push(Box::new(ExistingVar::default()));
            if expr_context != ExprContext::Default {
                rules.push(Box::new(AssignToFunction::default()));
            } else {
                // no need to check for built-in, they don't have dots
                rules.push(Box::new(VarAsUserDefinedFunctionCall::default()));
            }
            rules.push(Box::new(ExistingConst::new_local()));
            rules.push(Box::new(ExistingConst::new_recursive()));
            for mut rule in rules {
                if rule.can_handle(ctx, &folded_name) {
                    return rule.resolve_no_implicits(ctx, folded_name, pos);
                }
            }
        }

        // it is not a folded name, either it is a property of a known variable or expression,
        // or we need to introduce a new implicit var with a dot
        let unboxed_left_side = *left_side;
        let (
            Locatable {
                element: resolved_left_side,
                ..
            },
            implicit_left_side,
        ) = ctx.on_expression(
            unboxed_left_side.at(pos),
            ExprContext::ResolvingPropertyOwner,
        )?;

        // functions cannot return udf so no need to check them
        match &resolved_left_side {
            Expression::Variable(
                _name,
                VariableInfo {
                    expression_type, ..
                },
            ) => {
                let temp_expression_type = expression_type.clone();
                existing_property_expression_type(
                    ctx,
                    resolved_left_side,
                    &temp_expression_type,
                    property_name,
                    implicit_left_side,
                    pos,
                    true,
                )
            }
            Expression::ArrayElement(
                _name,
                _indices,
                VariableInfo {
                    expression_type, ..
                },
            ) => {
                let temp_expression_type = expression_type.clone();
                existing_property_expression_type(
                    ctx,
                    resolved_left_side,
                    &temp_expression_type,
                    property_name,
                    implicit_left_side,
                    pos,
                    false,
                )
            }
            Expression::Property(_left_side, _name, expr_type) => {
                let temp_expression_type = expr_type.clone();
                existing_property_expression_type(
                    ctx,
                    resolved_left_side,
                    &temp_expression_type,
                    property_name,
                    implicit_left_side,
                    pos,
                    false,
                )
            }
            _ => {
                // this cannot possibly have a dot property
                return Err(QError::TypeMismatch).with_err_at(pos);
            }
        }
    }

    fn try_fold(left_side: &Expression, property_name: Name) -> Option<Name> {
        match left_side.fold_name() {
            Some(left_side_name) => left_side_name.try_concat_name(property_name),
            _ => None,
        }
    }

    fn existing_property_expression_type(
        ctx: &mut Context,
        resolved_left_side: Expression,
        expression_type: &ExpressionType,
        property_name: Name,
        implicit_left_side: Vec<QualifiedNameNode>,
        pos: Location,
        allow_unresolved: bool,
    ) -> R {
        match expression_type {
            ExpressionType::UserDefined(user_defined_type_name) => {
                existing_property_user_defined_type_name(
                    ctx,
                    resolved_left_side,
                    user_defined_type_name,
                    property_name,
                    implicit_left_side,
                    pos,
                )
            }
            ExpressionType::Unresolved => {
                if allow_unresolved {
                    match try_fold(&resolved_left_side, property_name) {
                        Some(folded_name) => Ok(add_as_new_implicit_var(ctx, folded_name, pos)),
                        _ => Err(QError::TypeMismatch).with_err_at(pos),
                    }
                } else {
                    Err(QError::TypeMismatch).with_err_at(pos)
                }
            }
            _ => Err(QError::TypeMismatch).with_err_at(pos),
        }
    }

    fn existing_property_user_defined_type_name(
        ctx: &Context,
        resolved_left_side: Expression,
        user_defined_type_name: &BareName,
        property_name: Name,
        implicit_left_side: Vec<QualifiedNameNode>,
        pos: Location,
    ) -> R {
        match ctx.user_defined_types.get(user_defined_type_name) {
            Some(user_defined_type) => existing_property_user_defined_type(
                resolved_left_side,
                user_defined_type,
                property_name,
                implicit_left_side,
                pos,
            ),
            _ => Err(QError::TypeNotDefined).with_err_at(pos),
        }
    }

    fn existing_property_user_defined_type(
        resolved_left_side: Expression,
        user_defined_type: &UserDefinedType,
        property_name: Name,
        implicit_left_side: Vec<QualifiedNameNode>,
        pos: Location,
    ) -> R {
        match user_defined_type.demand_element_by_name(&property_name) {
            Ok(element_type) => existing_property_element_type(
                resolved_left_side,
                element_type,
                property_name,
                implicit_left_side,
                pos,
            ),
            Err(e) => Err(e).with_err_at(pos),
        }
    }

    fn existing_property_element_type(
        resolved_left_side: Expression,
        element_type: &ElementType,
        property_name: Name,
        implicit_left_side: Vec<QualifiedNameNode>,
        pos: Location,
    ) -> R {
        Ok((
            Expression::Property(
                Box::new(resolved_left_side),
                property_name.un_qualify(),
                element_type.expression_type(),
            )
            .at(pos),
            implicit_left_side,
        ))
    }
}

pub mod function_v2 {
    use super::*;
    pub fn convert(
        ctx: &mut Context,
        name: Name,
        args: ExpressionNodes,
        expr_context: ExprContext,
        pos: Location,
    ) -> R {
        let mut rules: Vec<Box<dyn FuncResolve>> = vec![];
        // these go first because they're allowed to have no arguments
        rules.push(Box::new(ExistingArrayWithParenthesis::default()));
        for mut rule in rules {
            if rule.can_handle(ctx, &name) {
                return rule.resolve(ctx, name, args, expr_context, pos);
            }
        }

        // now validate we have arguments
        if args.is_empty() {
            return Err(QError::syntax_error(
                "Cannot have function call without arguments",
            ))
            .with_err_at(pos);
        }
        // continue with built-in/user defined functions
        resolve_function(ctx, name, args, pos)
    }

    fn resolve_function(ctx: &mut Context, name: Name, args: ExpressionNodes, pos: Location) -> R {
        // convert args
        let (converted_args, implicit_vars) = ctx.on_expressions(args, ExprContext::Parameter)?;
        // is it built-in function?
        let converted_expr = match Option::<BuiltInFunction>::try_from(&name).with_err_at(pos)? {
            Some(built_in_function) => {
                Expression::BuiltInFunctionCall(built_in_function, converted_args)
            }
            _ => {
                let converted_name: Name = match ctx.function_qualifier(name.bare_name()) {
                    Some(q) => name.qualify(q),
                    _ => ctx.resolve_name_to_name(name),
                };
                Expression::FunctionCall(converted_name, converted_args)
            }
        };
        Ok((converted_expr.at(pos), implicit_vars))
    }

    trait FuncResolve {
        fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool;

        fn resolve(
            &self,
            ctx: &mut Context,
            name: Name,
            args: ExpressionNodes,
            expr_context: ExprContext,
            pos: Location,
        ) -> Result<O, QErrorNode>;
    }

    #[derive(Default)]
    struct ExistingArrayWithParenthesis {
        var_info: Option<VariableInfo>,
    }

    impl ExistingArrayWithParenthesis {
        fn is_array(&self) -> bool {
            match &self.var_info {
                Some(var_info) => match &var_info.expression_type {
                    ExpressionType::Array(_) => true,
                    _ => false,
                },
                _ => false,
            }
        }
    }

    impl FuncResolve for ExistingArrayWithParenthesis {
        fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool {
            let bare_name = name.bare_name();
            self.var_info = ctx
                .names
                .get_extended_var_recursively(bare_name)
                .map(Clone::clone);
            if self.var_info.is_some() {
                self.is_array()
            } else {
                let qualifier: TypeQualifier = ctx.resolve_name_to_qualifier(&name);
                self.var_info = ctx
                    .names
                    .get_compact_var_recursively(bare_name, qualifier)
                    .map(Clone::clone);
                self.is_array()
            }
        }

        fn resolve(
            &self,
            ctx: &mut Context,
            name: Name,
            args: ExpressionNodes,
            expr_context: ExprContext,
            pos: Location,
        ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
            // convert args
            let (converted_args, implicit_vars) = ctx.on_expressions(args, expr_context)?;
            // convert name
            let VariableInfo {
                expression_type,
                shared,
            } = self.var_info.clone().unwrap();
            match expression_type {
                ExpressionType::Array(element_type) => {
                    let converted_name = element_type.qualify_name(name).with_err_at(pos)?;
                    // create result
                    let result_expr = Expression::ArrayElement(
                        converted_name,
                        converted_args,
                        VariableInfo {
                            expression_type: element_type.as_ref().clone(),
                            shared,
                        },
                    );
                    Ok((result_expr.at(pos), implicit_vars))
                }
                _ => Err(QError::ArrayNotDefined).with_err_at(pos),
            }
        }
    }
}
