use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::linter::converter::converter::Context;
use crate::linter::converter::traits::{Convertible, FromParentContext};
use crate::parser::*;
use crate::variant::Variant;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExprContext {
    Default,
    Assignment,
    Parameter,

    /// Used in resolving left-side of property expressions
    ResolvingPropertyOwner,
}

impl Default for ExprContext {
    fn default() -> Self {
        Self::Default
    }
}

pub struct ExprState<'a> {
    ctx: &'a mut Context,
    expr_context: ExprContext,
}

impl<'a> ExprState<'a> {
    pub fn new(ctx: &'a mut Context, expr_context: ExprContext) -> Self {
        Self { ctx, expr_context }
    }
}

impl<'a> FromParentContext<'a, Context, ExprContext> for ExprState<'a> {
    fn create_from_parent_context(parent: &'a mut Context, value: ExprContext) -> Self {
        Self::new(parent, value)
    }
}

impl<'a> Convertible<ExprState<'a>> for ExpressionNode {
    fn convert(self, ctx: &mut ExprState<'a>) -> Result<Self, QErrorNode> {
        let Locatable { element: expr, pos } = self;
        match expr {
            // literals
            Expression::SingleLiteral(f) => ok_locatable(Expression::SingleLiteral(f), pos),
            Expression::DoubleLiteral(d) => ok_locatable(Expression::DoubleLiteral(d), pos),
            Expression::StringLiteral(s) => ok_locatable(Expression::StringLiteral(s), pos),
            Expression::IntegerLiteral(i) => ok_locatable(Expression::IntegerLiteral(i), pos),
            Expression::LongLiteral(l) => ok_locatable(Expression::LongLiteral(l), pos),
            // parenthesis
            Expression::Parenthesis(box_child) => parenthesis::convert(ctx, box_child, pos),
            // unary
            Expression::UnaryExpression(unary_operator, box_child) => {
                unary::convert(ctx, unary_operator, box_child, pos)
            }
            // binary
            Expression::BinaryExpression(binary_operator, left, right, _expr_type) => {
                binary::convert(ctx, binary_operator, left, right, pos)
            }
            // variables
            Expression::Variable(name, variable_info) => {
                variable::convert(ctx, name, variable_info, pos)
            }
            Expression::ArrayElement(_name, _indices, _variable_info) => {
                panic!(
                    "Parser is not supposed to produce any ArrayElement expressions, only FunctionCall"
                )
            }
            Expression::Property(box_left_side, property_name, _expr_type) => {
                property::convert(ctx, box_left_side, property_name, pos)
            }
            // function call
            Expression::FunctionCall(name, args) => function::convert(ctx, name, args, pos),
            Expression::BuiltInFunctionCall(built_in_function, args) => {
                built_in_function::convert(ctx.ctx, built_in_function, args, pos)
            }
        }
    }
}

// TODO #[deprecated] use a blanket Locatable impl
fn ok_locatable(expr: Expression, pos: Location) -> Result<ExpressionNode, QErrorNode> {
    Ok(expr.at(pos))
}

mod parenthesis {
    use super::*;

    pub fn convert(
        ctx: &mut ExprState,
        box_child: Box<ExpressionNode>,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        // convert child (recursion)
        let converted_child = (*box_child).convert(ctx)?;
        let parenthesis_expr = Expression::Parenthesis(Box::new(converted_child));
        Ok(parenthesis_expr.at(pos))
    }
}

mod unary {
    use super::*;
    pub fn convert(
        ctx: &mut ExprState,
        unary_operator: UnaryOperator,
        box_child: Box<ExpressionNode>,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        // convert child (recursion)
        let converted_child = (*box_child).convert(ctx)?;
        // ensure operator applies to converted expr
        let converted_expr_type = converted_child.as_ref().expression_type();
        if is_applicable_to_expr_type(&converted_expr_type) {
            let unary_expr = Expression::UnaryExpression(unary_operator, Box::new(converted_child));
            Ok(unary_expr.at(pos))
        } else {
            Err(QError::TypeMismatch).with_err_at(&converted_child)
        }
    }

    fn is_applicable_to_expr_type(expr_type: &ExpressionType) -> bool {
        match expr_type {
            ExpressionType::BuiltIn(TypeQualifier::BangSingle)
            | ExpressionType::BuiltIn(TypeQualifier::HashDouble)
            | ExpressionType::BuiltIn(TypeQualifier::PercentInteger)
            | ExpressionType::BuiltIn(TypeQualifier::AmpersandLong) => true,
            _ => false,
        }
    }
}

mod binary {
    use super::*;
    pub fn convert(
        ctx: &mut ExprState,
        binary_operator: Operator,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        let converted_left = (*left).convert(ctx)?;
        let converted_right = (*right).convert(ctx)?;
        let new_expr = Expression::binary(converted_left, converted_right, binary_operator)?;
        Ok(new_expr.at(pos))
    }
}

mod variable {
    use super::*;
    use crate::linter::type_resolver::{IntoQualified, IntoTypeQualifier};
    use crate::linter::HasSubs;

    pub fn convert(
        ctx: &mut ExprState,
        name: Name,
        variable_info: VariableInfo,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        // validation rules
        validate(ctx.ctx, &name, pos)?;

        // match existing
        let mut rules: Vec<Box<dyn VarResolve>> = vec![];
        rules.push(Box::new(ExistingVar::default()));
        rules.push(Box::new(ExistingConst::new_local()));
        if ctx.expr_context != ExprContext::Default {
            rules.push(Box::new(AssignToFunction::default()));
        } else {
            rules.push(Box::new(VarAsBuiltInFunctionCall::default()));
            rules.push(Box::new(VarAsUserDefinedFunctionCall::default()));
        }
        rules.push(Box::new(ExistingConst::new_recursive()));

        for mut rule in rules {
            if rule.can_handle(ctx.ctx, &name) {
                return rule.resolve(ctx.ctx, name, pos);
            }
        }

        if ctx.expr_context != ExprContext::ResolvingPropertyOwner {
            // add as new implicit
            Ok(add_as_new_implicit_var(ctx.ctx, name, pos))
        } else {
            // repack as unresolved
            Ok(Expression::Variable(name, variable_info).at(pos))
        }
    }

    fn validate(ctx: &Context, name: &Name, pos: Location) -> Result<(), QErrorNode> {
        if ctx.subs().contains_key(name.bare_name()) {
            return Err(QError::DuplicateDefinition).with_err_at(pos);
        }

        Ok(())
    }

    pub fn add_as_new_implicit_var(ctx: &mut Context, name: Name, pos: Location) -> ExpressionNode {
        let resolved_name = name.to_qualified(ctx);

        let bare_name = resolved_name.bare_name();
        let q = resolved_name.qualifier().expect("Should be resolved");
        ctx.names.insert(
            bare_name.clone(),
            &DimType::BuiltIn(q, BuiltInStyle::Compact),
            false,
            None,
        );

        let var_info = VariableInfo::new_built_in(q, false);
        ctx.names
            .add_implicit(resolved_name.clone().demand_qualified().at(pos));
        Expression::Variable(resolved_name, var_info).at(pos)
    }

    pub trait VarResolve {
        fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool;

        fn resolve(
            &self,
            ctx: &Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode>;
    }

    #[derive(Default)]
    pub struct ExistingVar {
        var_info: Option<VariableInfo>,
    }

    impl VarResolve for ExistingVar {
        fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool {
            let bare_name = name.bare_name();
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
            _ctx: &Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            let variable_info = self.var_info.clone().unwrap();
            let expression_type = &variable_info.expression_type;
            let converted_name = expression_type.qualify_name(name).with_err_at(pos)?;
            Ok(Expression::Variable(converted_name, variable_info).at(pos))
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
                    .get_const_value_recursively(name.bare_name())
                    .cloned()
            } else {
                ctx.names
                    .get_const_value_no_recursion(name.bare_name())
                    .cloned()
            };
            self.opt_v.is_some()
        }

        fn resolve(
            &self,
            _ctx: &Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            let v = self.opt_v.clone().unwrap();
            let q = TypeQualifier::try_from(&v).with_err_at(pos)?;
            if name.is_bare_or_of_type(q) {
                // resolve to literal expression
                let expr = Expression::from_constant(v).with_err_at(pos)?;
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

    impl VarResolve for AssignToFunction {
        fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool {
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
            ctx: &Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            let function_qualifier = self.function_qualifier.unwrap();
            if ctx.names.is_in_function(name.bare_name()) {
                let converted_name = name.try_qualify(function_qualifier).with_err_at(pos)?;
                let expr_type = ExpressionType::BuiltIn(function_qualifier);
                let expr = Expression::Variable(converted_name, VariableInfo::new_local(expr_type));
                Ok(expr.at(pos))
            } else {
                Err(QError::DuplicateDefinition).with_err_at(pos)
            }
        }
    }

    #[derive(Default)]
    pub struct VarAsBuiltInFunctionCall {
        built_in_function: Option<BuiltInFunction>,
    }

    impl VarResolve for VarAsBuiltInFunctionCall {
        fn can_handle(&mut self, _ctx: &Context, name: &Name) -> bool {
            self.built_in_function = name.bare_name().into();
            self.built_in_function.is_some()
        }

        fn resolve(
            &self,
            _ctx: &Context,
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

    impl VarResolve for VarAsUserDefinedFunctionCall {
        fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool {
            self.function_qualifier = ctx.function_qualifier(name.bare_name());
            self.function_qualifier.is_some()
        }

        fn resolve(
            &self,
            _ctx: &Context,
            name: Name,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            let q = self.function_qualifier.unwrap();
            let converted_name = name.try_qualify(q).with_err_at(pos)?;
            Ok(Expression::FunctionCall(converted_name, vec![]).at(pos))
        }
    }
}

mod property {
    use super::*;
    use crate::linter::converter::expr_rules::variable::{
        add_as_new_implicit_var, AssignToFunction, ExistingConst, ExistingVar,
        VarAsUserDefinedFunctionCall, VarResolve,
    };
    use crate::linter::HasUserDefinedTypes;

    pub fn convert(
        ctx: &mut ExprState,
        left_side: Box<Expression>,
        property_name: Name,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        // can we fold it into a name?
        let opt_folded_name = try_fold(&left_side, property_name.clone());
        if let Some(folded_name) = opt_folded_name {
            // checking out if we have an existing variable / const etc that contains a dot
            let mut rules: Vec<Box<dyn VarResolve>> = vec![];
            rules.push(Box::new(ExistingVar::default()));
            if ctx.expr_context != ExprContext::Default {
                rules.push(Box::new(AssignToFunction::default()));
            } else {
                // no need to check for built-in, they don't have dots
                rules.push(Box::new(VarAsUserDefinedFunctionCall::default()));
            }
            rules.push(Box::new(ExistingConst::new_local()));
            rules.push(Box::new(ExistingConst::new_recursive()));
            for mut rule in rules {
                if rule.can_handle(ctx.ctx, &folded_name) {
                    return rule.resolve(ctx.ctx, folded_name, pos);
                }
            }
        }

        // it is not a folded name, either it is a property of a known variable or expression,
        // or we need to introduce a new implicit var with a dot
        let unboxed_left_side = *left_side;
        let Locatable {
            element: resolved_left_side,
            ..
        } = unboxed_left_side
            .at(pos)
            .convert_in(ctx.ctx, ExprContext::ResolvingPropertyOwner)?;

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
                    ctx.ctx,
                    resolved_left_side,
                    &temp_expression_type,
                    property_name,
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
                    ctx.ctx,
                    resolved_left_side,
                    &temp_expression_type,
                    property_name,
                    pos,
                    false,
                )
            }
            Expression::Property(_left_side, _name, expr_type) => {
                let temp_expression_type = expr_type.clone();
                existing_property_expression_type(
                    ctx.ctx,
                    resolved_left_side,
                    &temp_expression_type,
                    property_name,
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
        pos: Location,
        allow_unresolved: bool,
    ) -> Result<ExpressionNode, QErrorNode> {
        match expression_type {
            ExpressionType::UserDefined(user_defined_type_name) => {
                existing_property_user_defined_type_name(
                    ctx,
                    resolved_left_side,
                    user_defined_type_name,
                    property_name,
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
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        match ctx.user_defined_types().get(user_defined_type_name) {
            Some(user_defined_type) => existing_property_user_defined_type(
                resolved_left_side,
                user_defined_type,
                property_name,
                pos,
            ),
            _ => Err(QError::TypeNotDefined).with_err_at(pos),
        }
    }

    fn existing_property_user_defined_type(
        resolved_left_side: Expression,
        user_defined_type: &UserDefinedType,
        property_name: Name,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        match user_defined_type.demand_element_by_name(&property_name) {
            Ok(element_type) => {
                existing_property_element_type(resolved_left_side, element_type, property_name, pos)
            }
            Err(e) => Err(e).with_err_at(pos),
        }
    }

    fn existing_property_element_type(
        resolved_left_side: Expression,
        element_type: &ElementType,
        property_name: Name,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        let bare_name = property_name.into();
        let property_name = Name::Bare(bare_name);
        Ok(Expression::Property(
            Box::new(resolved_left_side),
            property_name,
            element_type.expression_type(),
        )
        .at(pos))
    }
}

fn functions_must_have_arguments(args: &ExpressionNodes, pos: Location) -> Result<(), QErrorNode> {
    if args.is_empty() {
        Err(QError::FunctionNeedsArguments).with_err_at(pos)
    } else {
        Ok(())
    }
}

fn convert_function_args(
    ctx: &mut Context,
    args: ExpressionNodes,
) -> Result<ExpressionNodes, QErrorNode> {
    args.convert_in(ctx, ExprContext::Parameter)
}

mod built_in_function {
    use super::*;
    pub fn convert(
        ctx: &mut Context,
        built_in_function: BuiltInFunction,
        args: ExpressionNodes,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        functions_must_have_arguments(&args, pos)?;
        let converted_args = convert_function_args(ctx, args)?;
        let converted_expr = Expression::BuiltInFunctionCall(built_in_function, converted_args);
        Ok(converted_expr.at(pos))
    }
}

mod function {
    use super::*;
    use crate::linter::type_resolver::{IntoQualified, IntoTypeQualifier};
    pub fn convert(
        ctx: &mut ExprState,
        name: Name,
        args: ExpressionNodes,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        let mut rules: Vec<Box<dyn FuncResolve>> = vec![];
        // these go first because they're allowed to have no arguments
        rules.push(Box::new(ExistingArrayWithParenthesis::default()));
        for mut rule in rules {
            if rule.can_handle(ctx.ctx, &name) {
                return rule.resolve(ctx, name, args, pos);
            }
        }

        // now validate we have arguments
        functions_must_have_arguments(&args, pos)?;
        // continue with built-in/user defined functions
        resolve_function(ctx.ctx, name, args, pos)
    }

    fn resolve_function(
        ctx: &mut Context,
        name: Name,
        args: ExpressionNodes,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        // convert args
        let converted_args = convert_function_args(ctx, args)?;
        // is it built-in function?
        let converted_expr = match Option::<BuiltInFunction>::try_from(&name).with_err_at(pos)? {
            Some(built_in_function) => {
                Expression::BuiltInFunctionCall(built_in_function, converted_args)
            }
            _ => {
                let converted_name: Name = match ctx.function_qualifier(name.bare_name()) {
                    Some(function_qualifier) => {
                        name.try_qualify(function_qualifier).with_err_at(pos)?
                    }
                    _ => name.to_qualified(ctx),
                };
                Expression::FunctionCall(converted_name, converted_args)
            }
        };
        Ok(converted_expr.at(pos))
    }

    trait FuncResolve {
        fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool;

        fn resolve(
            &self,
            ctx: &mut ExprState,
            name: Name,
            args: ExpressionNodes,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode>;
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
                let qualifier = name.qualify(ctx);
                self.var_info = ctx
                    .names
                    .get_compact_var_recursively(bare_name, qualifier)
                    .map(Clone::clone);
                self.is_array()
            }
        }

        fn resolve(
            &self,
            ctx: &mut ExprState,
            name: Name,
            args: ExpressionNodes,
            pos: Location,
        ) -> Result<ExpressionNode, QErrorNode> {
            // convert args
            let converted_args = args.convert(ctx)?;
            // convert name
            let VariableInfo {
                expression_type,
                shared,
                redim_info,
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
                            redim_info,
                        },
                    );
                    Ok(result_expr.at(pos))
                }
                _ => Err(QError::ArrayNotDefined).with_err_at(pos),
            }
        }
    }
}
