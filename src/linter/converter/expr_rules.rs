use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::linter::converter::converter::Context;
use crate::linter::converter::traits::Convertible;
use crate::parser::*;
use crate::variant::Variant;
use expr_state::ExprState;
use pos_expr_state::PosExprState;
use std::convert::TryFrom;

/// Indicates the context in which an expression is being resolved.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExprContext {
    /// Default context (typically r-side expression)
    Default,

    /// Assignment (typically l-side expression)
    Assignment,

    /// Function or sub argument
    Argument,

    /// Used in resolving left-side of property expressions
    ResolvingPropertyOwner,
}

impl Default for ExprContext {
    fn default() -> Self {
        Self::Default
    }
}

mod expr_state {
    use crate::linter::converter::converter::Context;
    use crate::linter::converter::expr_rules::ExprContext;
    use crate::linter::converter::traits::FromParentContext;
    use std::ops::{Deref, DerefMut};

    /// A context that is used when converting an [ExpressionNode].
    /// Enhances the parent [Context] with an [ExprContext].
    pub struct ExprState<'a> {
        ctx: &'a mut Context,
        expr_context: ExprContext,
    }

    impl<'a> ExprState<'a> {
        pub fn new(ctx: &'a mut Context, expr_context: ExprContext) -> Self {
            Self { ctx, expr_context }
        }

        pub fn expr_context(&self) -> ExprContext {
            self.expr_context
        }
    }

    impl<'a> FromParentContext<'a, Context, ExprContext> for ExprState<'a> {
        fn create_from_parent_context(parent: &'a mut Context, value: ExprContext) -> Self {
            Self::new(parent, value)
        }
    }

    impl<'a> Deref for ExprState<'a> {
        type Target = Context;

        fn deref(&self) -> &Self::Target {
            self.ctx
        }
    }

    impl<'a> DerefMut for ExprState<'a> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.ctx
        }
    }
}

mod pos_expr_state {

    //
    // PosExprState
    //

    use crate::common::{HasLocation, Location};
    use crate::linter::converter::expr_rules::expr_state::ExprState;
    use crate::linter::converter::traits::FromParentContext;
    use std::ops::{Deref, DerefMut};

    pub struct PosExprState<'a, 'b> {
        ctx: &'a mut ExprState<'b>,
        pos: Location,
    }

    impl<'a, 'b> PosExprState<'a, 'b> {
        pub fn new(ctx: &'a mut ExprState<'b>, pos: Location) -> Self {
            Self { ctx, pos }
        }
    }

    impl<'a, 'b> FromParentContext<'a, ExprState<'b>, Location> for PosExprState<'a, 'b> {
        fn create_from_parent_context(parent: &'a mut ExprState<'b>, value: Location) -> Self {
            Self::new(parent, value)
        }
    }

    impl<'a, 'b> Deref for PosExprState<'a, 'b> {
        type Target = ExprState<'b>;

        fn deref(&self) -> &Self::Target {
            self.ctx
        }
    }

    impl<'a, 'b> DerefMut for PosExprState<'a, 'b> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.ctx
        }
    }

    impl<'a, 'b> HasLocation for PosExprState<'a, 'b> {
        fn pos(&self) -> Location {
            self.pos
        }
    }
}

//
// ExpressionNode Convertible
//

impl<'a> Convertible<ExprState<'a>> for ExpressionNode {
    fn convert(self, ctx: &mut ExprState<'a>) -> Result<Self, QErrorNode> {
        let Locatable { element: expr, pos } = self;
        match expr.convert_in(ctx, pos) {
            Ok(expr) => Ok(expr.at(pos)),
            Err(err) => Err(err.patch_pos(pos)),
        }
    }
}

impl<'a> Convertible<ExprState<'a>> for Box<ExpressionNode> {
    fn convert(self, ctx: &mut ExprState<'a>) -> Result<Self, QErrorNode> {
        let unboxed = *self;
        unboxed.convert(ctx).map(Box::new)
    }
}

//
// Expression Convertible
//

impl<'a, 'b> Convertible<PosExprState<'a, 'b>> for Expression {
    fn convert(self, ctx: &mut PosExprState<'a, 'b>) -> Result<Self, QErrorNode> {
        match self {
            // literals
            Expression::SingleLiteral(_)
            | Expression::DoubleLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::IntegerLiteral(_)
            | Expression::LongLiteral(_) => Ok(self),
            // parenthesis
            Expression::Parenthesis(box_child) => {
                box_child.convert(ctx).map(Expression::Parenthesis)
            }
            // unary
            Expression::UnaryExpression(unary_operator, box_child) => {
                unary::convert(ctx, unary_operator, box_child)
            }
            // binary
            Expression::BinaryExpression(binary_operator, left, right, _expr_type) => {
                binary::convert(ctx, binary_operator, left, right)
            }
            // variables
            Expression::Variable(name, variable_info) => {
                variable::convert(ctx, name, variable_info)
            }
            Expression::ArrayElement(_name, _indices, _variable_info) => {
                panic!(
                    "Parser is not supposed to produce any ArrayElement expressions, only FunctionCall"
                )
            }
            Expression::Property(box_left_side, property_name, _expr_type) => {
                property::convert(ctx, box_left_side, property_name)
            }
            // function call
            Expression::FunctionCall(name, args) => function::convert(ctx, name, args),
            Expression::BuiltInFunctionCall(built_in_function, args) => {
                built_in_function::convert(ctx, built_in_function, args)
            }
        }
    }
}

mod unary {
    use super::*;
    pub fn convert(
        ctx: &mut PosExprState,
        unary_operator: UnaryOperator,
        box_child: Box<ExpressionNode>,
    ) -> Result<Expression, QErrorNode> {
        // convert child (recursion)
        let converted_child = (*box_child).convert(ctx)?;
        // ensure operator applies to converted expr
        let converted_expr_type = converted_child.as_ref().expression_type();
        if is_applicable_to_expr_type(&converted_expr_type) {
            let unary_expr = Expression::UnaryExpression(unary_operator, Box::new(converted_child));
            Ok(unary_expr)
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
        ctx: &mut PosExprState,
        binary_operator: Operator,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    ) -> Result<Expression, QErrorNode> {
        let converted_left = (*left).convert(ctx)?;
        let converted_right = (*right).convert(ctx)?;
        let new_expr = Expression::binary(converted_left, converted_right, binary_operator)?;
        Ok(new_expr)
    }
}

mod variable {
    use super::*;
    use crate::linter::type_resolver::{IntoQualified, IntoTypeQualifier};
    use crate::linter::HasSubs;

    pub fn convert(
        ctx: &mut PosExprState,
        name: Name,
        variable_info: VariableInfo,
    ) -> Result<Expression, QErrorNode> {
        // validation rules
        validate(ctx, &name)?;

        // match existing
        let mut rules: Vec<Box<dyn VarResolve>> = vec![];
        rules.push(Box::new(ExistingVar::default()));
        rules.push(Box::new(ExistingConst::new_local()));
        if ctx.expr_context() != ExprContext::Default {
            rules.push(Box::new(AssignToFunction::default()));
        } else {
            rules.push(Box::new(VarAsBuiltInFunctionCall::default()));
            rules.push(Box::new(VarAsUserDefinedFunctionCall::default()));
        }
        rules.push(Box::new(ExistingConst::new_recursive()));

        for mut rule in rules {
            if rule.can_handle(ctx, &name) {
                return rule.resolve(ctx, name);
            }
        }

        if ctx.expr_context() != ExprContext::ResolvingPropertyOwner {
            // add as new implicit
            Ok(add_as_new_implicit_var(ctx, name))
        } else {
            // repack as unresolved
            Ok(Expression::Variable(name, variable_info))
        }
    }

    fn validate(ctx: &Context, name: &Name) -> Result<(), QErrorNode> {
        if ctx.subs().contains_key(name.bare_name()) {
            return Err(QError::DuplicateDefinition).with_err_no_pos();
        }

        Ok(())
    }

    pub fn add_as_new_implicit_var(ctx: &mut PosExprState, name: Name) -> Expression {
        let resolved_name = {
            let temp: &ExprState = &ctx;
            let temp: &Context = &temp;
            name.to_qualified(temp)
        };

        let bare_name = resolved_name.bare_name();
        let q = resolved_name.qualifier().expect("Should be resolved");
        ctx.names.insert(
            bare_name.clone(),
            &DimType::BuiltIn(q, BuiltInStyle::Compact),
            false,
            None,
        );

        let var_info = VariableInfo::new_built_in(q, false);
        let pos = ctx.pos();
        ctx.names
            .add_implicit(resolved_name.clone().demand_qualified().at(pos));
        Expression::Variable(resolved_name, var_info)
    }

    pub trait VarResolve {
        fn can_handle(&mut self, ctx: &Context, name: &Name) -> bool;

        fn resolve(&self, ctx: &PosExprState, name: Name) -> Result<Expression, QErrorNode>;
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

        fn resolve(&self, _ctx: &PosExprState, name: Name) -> Result<Expression, QErrorNode> {
            let variable_info = self.var_info.clone().unwrap();
            let expression_type = &variable_info.expression_type;
            let converted_name = expression_type.qualify_name(name).with_err_no_pos()?;
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
                    .get_const_value_recursively(name.bare_name())
                    .cloned()
            } else {
                ctx.names
                    .get_const_value_no_recursion(name.bare_name())
                    .cloned()
            };
            self.opt_v.is_some()
        }

        fn resolve(&self, _ctx: &PosExprState, name: Name) -> Result<Expression, QErrorNode> {
            let v = self.opt_v.clone().unwrap();
            let q = TypeQualifier::try_from(&v).with_err_no_pos()?;
            if name.is_bare_or_of_type(q) {
                // resolve to literal expression
                Expression::from_constant(v).with_err_no_pos()
            } else {
                Err(QError::DuplicateDefinition).with_err_no_pos()
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

        fn resolve(&self, ctx: &PosExprState, name: Name) -> Result<Expression, QErrorNode> {
            let function_qualifier = self.function_qualifier.unwrap();
            if ctx.names.is_in_function(name.bare_name()) {
                let converted_name = name.try_qualify(function_qualifier).with_err_no_pos()?;
                let expr_type = ExpressionType::BuiltIn(function_qualifier);
                let expr = Expression::Variable(converted_name, VariableInfo::new_local(expr_type));
                Ok(expr)
            } else {
                Err(QError::DuplicateDefinition).with_err_no_pos()
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

        fn resolve(&self, _ctx: &PosExprState, name: Name) -> Result<Expression, QErrorNode> {
            match Option::<BuiltInFunction>::try_from(&name).with_err_no_pos()? {
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
            self.function_qualifier = ctx.function_qualifier(name.bare_name());
            self.function_qualifier.is_some()
        }

        fn resolve(&self, _ctx: &PosExprState, name: Name) -> Result<Expression, QErrorNode> {
            let q = self.function_qualifier.unwrap();
            let converted_name = name.try_qualify(q).with_err_no_pos()?;
            Ok(Expression::FunctionCall(converted_name, vec![]))
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
        ctx: &mut PosExprState,
        left_side: Box<Expression>,
        property_name: Name,
    ) -> Result<Expression, QErrorNode> {
        // can we fold it into a name?
        let opt_folded_name = try_fold(&left_side, property_name.clone());
        if let Some(folded_name) = opt_folded_name {
            // checking out if we have an existing variable / const etc that contains a dot
            let mut rules: Vec<Box<dyn VarResolve>> = vec![];
            rules.push(Box::new(ExistingVar::default()));
            if ctx.expr_context() != ExprContext::Default {
                rules.push(Box::new(AssignToFunction::default()));
            } else {
                // no need to check for built-in, they don't have dots
                rules.push(Box::new(VarAsUserDefinedFunctionCall::default()));
            }
            rules.push(Box::new(ExistingConst::new_local()));
            rules.push(Box::new(ExistingConst::new_recursive()));
            for mut rule in rules {
                if rule.can_handle(ctx, &folded_name) {
                    return rule.resolve(ctx, folded_name);
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
            .at(ctx.pos())
            .convert_in(ctx, ExprContext::ResolvingPropertyOwner)?;

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
                    false,
                )
            }
            _ => {
                // this cannot possibly have a dot property
                Err(QError::TypeMismatch).with_err_no_pos()
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
        ctx: &mut PosExprState,
        resolved_left_side: Expression,
        expression_type: &ExpressionType,
        property_name: Name,
        allow_unresolved: bool,
    ) -> Result<Expression, QErrorNode> {
        match expression_type {
            ExpressionType::UserDefined(user_defined_type_name) => {
                existing_property_user_defined_type_name(
                    ctx,
                    resolved_left_side,
                    user_defined_type_name,
                    property_name,
                )
            }
            ExpressionType::Unresolved => {
                if allow_unresolved {
                    match try_fold(&resolved_left_side, property_name) {
                        Some(folded_name) => Ok(add_as_new_implicit_var(ctx, folded_name)),
                        _ => Err(QError::TypeMismatch).with_err_no_pos(),
                    }
                } else {
                    Err(QError::TypeMismatch).with_err_no_pos()
                }
            }
            _ => Err(QError::TypeMismatch).with_err_no_pos(),
        }
    }

    fn existing_property_user_defined_type_name(
        ctx: &Context,
        resolved_left_side: Expression,
        user_defined_type_name: &BareName,
        property_name: Name,
    ) -> Result<Expression, QErrorNode> {
        match ctx.user_defined_types().get(user_defined_type_name) {
            Some(user_defined_type) => existing_property_user_defined_type(
                resolved_left_side,
                user_defined_type,
                property_name,
            ),
            _ => Err(QError::TypeNotDefined).with_err_no_pos(),
        }
    }

    fn existing_property_user_defined_type(
        resolved_left_side: Expression,
        user_defined_type: &UserDefinedType,
        property_name: Name,
    ) -> Result<Expression, QErrorNode> {
        match user_defined_type.demand_element_by_name(&property_name) {
            Ok(element_type) => {
                existing_property_element_type(resolved_left_side, element_type, property_name)
            }
            Err(e) => Err(e).with_err_no_pos(),
        }
    }

    fn existing_property_element_type(
        resolved_left_side: Expression,
        element_type: &ElementType,
        property_name: Name,
    ) -> Result<Expression, QErrorNode> {
        let bare_name = property_name.into();
        let property_name = Name::Bare(bare_name);
        Ok(Expression::Property(
            Box::new(resolved_left_side),
            property_name,
            element_type.expression_type(),
        ))
    }
}

fn functions_must_have_arguments(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.is_empty() {
        Err(QError::FunctionNeedsArguments).with_err_no_pos()
    } else {
        Ok(())
    }
}

fn convert_function_args(
    ctx: &mut Context,
    args: ExpressionNodes,
) -> Result<ExpressionNodes, QErrorNode> {
    args.convert_in(ctx, ExprContext::Argument)
}

mod built_in_function {
    use super::*;
    pub fn convert(
        ctx: &mut Context,
        built_in_function: BuiltInFunction,
        args: ExpressionNodes,
    ) -> Result<Expression, QErrorNode> {
        functions_must_have_arguments(&args)?;
        let converted_args = convert_function_args(ctx, args)?;
        let converted_expr = Expression::BuiltInFunctionCall(built_in_function, converted_args);
        Ok(converted_expr)
    }
}

mod function {
    use super::*;
    use crate::linter::type_resolver::{IntoQualified, IntoTypeQualifier};
    pub fn convert(
        ctx: &mut PosExprState,
        name: Name,
        args: ExpressionNodes,
    ) -> Result<Expression, QErrorNode> {
        let mut rules: Vec<Box<dyn FuncResolve>> = vec![];
        // these go first because they're allowed to have no arguments
        rules.push(Box::new(ExistingArrayWithParenthesis::default()));
        for mut rule in rules {
            if rule.can_handle(ctx, &name) {
                return rule.resolve(ctx, name, args);
            }
        }

        // now validate we have arguments
        functions_must_have_arguments(&args)?;
        // continue with built-in/user defined functions
        resolve_function(ctx, name, args)
    }

    fn resolve_function(
        ctx: &mut Context,
        name: Name,
        args: ExpressionNodes,
    ) -> Result<Expression, QErrorNode> {
        // convert args
        let converted_args = convert_function_args(ctx, args)?;
        // is it built-in function?
        let converted_expr = match Option::<BuiltInFunction>::try_from(&name).with_err_no_pos()? {
            Some(built_in_function) => {
                Expression::BuiltInFunctionCall(built_in_function, converted_args)
            }
            _ => {
                let converted_name: Name = match ctx.function_qualifier(name.bare_name()) {
                    Some(function_qualifier) => {
                        name.try_qualify(function_qualifier).with_err_no_pos()?
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
            ctx: &mut PosExprState,
            name: Name,
            args: ExpressionNodes,
        ) -> Result<Expression, QErrorNode>;
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
            ctx: &mut PosExprState,
            name: Name,
            args: ExpressionNodes,
        ) -> Result<Expression, QErrorNode> {
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
                    let converted_name = element_type.qualify_name(name).with_err_no_pos()?;
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
                _ => Err(QError::ArrayNotDefined).with_err_no_pos(),
            }
        }
    }
}
