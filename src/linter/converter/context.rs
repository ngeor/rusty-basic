use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;

use crate::common::{AtLocation, CaseInsensitiveString, Locatable, QErrorNode};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::*;
use crate::variant::Variant;

/*

Naming rules

1. It is possible to have multiple compact variables

e.g. A = 3.14 (resolves as A! by the default rules), A$ = "hello", A% = 1

2. A constant can be referenced either bare or by its correct qualifier

2b. A constant cannot co-exist with other symbols of the same name

3. A bare constant gets its qualifier from the expression and not from the type resolver

4. A constant in a subprogram can override a global constant

5. An extended variable can be referenced either bare or by its correct qualifier
5b. An extended variable cannot co-exist with other symbols of the same name
*/

pub struct Context<'a> {
    functions: &'a FunctionMap,
    subs: &'a SubMap,
    user_defined_types: &'a UserDefinedTypes,
    resolver: Rc<RefCell<TypeResolverImpl>>,
    names: Names,
    names_without_dot: HashSet<BareName>,
}

struct Names {
    compact_name_set: HashMap<BareName, HashSet<TypeQualifier>>,
    compact_names: HashMap<Name, VariableInfo>,
    extended_names: HashMap<BareName, VariableInfo>,
    constants: HashMap<BareName, Variant>,
    current_function_name: Option<BareName>,
    parent: Option<Box<Names>>,
}

impl Names {
    pub fn new(parent: Option<Box<Self>>, current_function_name: Option<BareName>) -> Self {
        Self {
            compact_name_set: HashMap::new(),
            compact_names: HashMap::new(),
            extended_names: HashMap::new(),
            constants: HashMap::new(),
            current_function_name,
            parent,
        }
    }

    pub fn new_root() -> Self {
        Self::new(None, None)
    }

    pub fn contains_local_var_or_local_const(&self, bare_name: &BareName) -> bool {
        self.compact_name_set.contains_key(bare_name)
            || self.extended_names.contains_key(bare_name)
            || self.constants.contains_key(bare_name)
    }

    /// Checks if a new compact variable can be introduced for the given name and qualifier.
    /// This is allowed if the given name is not yet known, or if it is known as a compact
    /// name and the qualifier hasn't been used yet.
    pub fn can_accept_compact(&self, bare_name: &BareName, qualifier: TypeQualifier) -> bool {
        if self.extended_names.contains_key(bare_name) || self.constants.contains_key(bare_name) {
            false
        } else {
            match self.compact_name_set.get(bare_name) {
                Some(qualifiers) => !qualifiers.contains(&qualifier),
                _ => true,
            }
        }
    }

    fn get_local_compact_var(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        match self.compact_name_set.get(bare_name) {
            Some(qualifiers) => {
                if qualifiers.contains(&qualifier) {
                    let name = Name::new(bare_name.clone(), Some(qualifier));
                    self.compact_names.get(&name)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_compact_var_recursively(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        match self.get_local_compact_var(bare_name, qualifier) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => {
                    parent_names.get_compact_shared_var_recursively(bare_name, qualifier)
                }
                _ => None,
            },
        }
    }

    fn get_compact_shared_var_recursively(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        match Self::require_shared(self.get_local_compact_var(bare_name, qualifier)) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => {
                    parent_names.get_compact_shared_var_recursively(bare_name, qualifier)
                }
                _ => None,
            },
        }
    }

    fn require_shared(opt_variable_info: Option<&VariableInfo>) -> Option<&VariableInfo> {
        match opt_variable_info {
            Some(variable_info) => {
                if variable_info.shared {
                    opt_variable_info
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn get_local_extended_var(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        self.extended_names.get(bare_name)
    }

    pub fn get_extended_var_recursively(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        match self.get_local_extended_var(bare_name) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => parent_names.get_extended_shared_var_recursively(bare_name),
                _ => None,
            },
        }
    }

    fn get_extended_shared_var_recursively(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        match Self::require_shared(self.get_local_extended_var(bare_name)) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => parent_names.get_extended_shared_var_recursively(bare_name),
                _ => None,
            },
        }
    }

    pub fn contains_const(&self, bare_name: &BareName) -> bool {
        self.constants.contains_key(bare_name)
    }

    pub fn get_const_value_no_recursion(&self, bare_name: &BareName) -> Option<&Variant> {
        self.constants.get(bare_name)
    }

    pub fn get_const_value_recursively(&self, bare_name: &BareName) -> Option<&Variant> {
        match self.constants.get(bare_name) {
            Some(v) => Some(v),
            _ => {
                if let Some(boxed_parent) = &self.parent {
                    boxed_parent.get_const_value_recursively(bare_name)
                } else {
                    None
                }
            }
        }
    }

    pub fn contains_const_recursively(&self, bare_name: &BareName) -> bool {
        if self.contains_const(bare_name) {
            true
        } else if let Some(boxed_parent) = &self.parent {
            boxed_parent.contains_const_recursively(bare_name)
        } else {
            false
        }
    }

    pub fn insert_compact(
        &mut self,
        bare_name: BareName,
        q: TypeQualifier,
        variable_context: VariableInfo,
    ) {
        self.compact_names
            .insert(Name::new(bare_name.clone(), Some(q)), variable_context);
        match self.compact_name_set.get_mut(&bare_name) {
            Some(s) => {
                s.insert(q);
            }
            None => {
                let mut s: HashSet<TypeQualifier> = HashSet::new();
                s.insert(q);
                self.compact_name_set.insert(bare_name, s);
            }
        }
    }

    pub fn insert_extended(&mut self, bare_name: BareName, variable_context: VariableInfo) {
        self.extended_names.insert(bare_name, variable_context);
    }

    pub fn insert_const(&mut self, bare_name: BareName, v: Variant) {
        self.constants.insert(bare_name, v);
    }
}

impl ConstValueResolver for Names {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        match self.constants.get(name) {
            Some(v) => Some(v),
            _ => match &self.parent {
                Some(boxed_parent) => boxed_parent.get_resolved_constant(name),
                _ => None,
            },
        }
    }
}

impl<'a> TypeResolver for Context<'a> {
    fn resolve_char(&self, ch: char) -> TypeQualifier {
        self.resolver.borrow().resolve_char(ch)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExprContext {
    Default,
    Assignment,
    Parameter,
    ResolvingPropertyOwner,
}

impl<'a> Context<'a> {
    pub fn new(
        functions: &'a FunctionMap,
        subs: &'a SubMap,
        user_defined_types: &'a UserDefinedTypes,
        resolver: Rc<RefCell<TypeResolverImpl>>,
    ) -> Self {
        Self {
            functions,
            subs,
            user_defined_types,
            resolver,
            names: Names::new_root(),
            names_without_dot: HashSet::new(),
        }
    }

    pub fn push_sub_context(
        &mut self,
        params: ParamNameNodes,
    ) -> Result<ParamNameNodes, QErrorNode> {
        let temp_dummy = Names::new_root();
        let old_names = std::mem::replace(&mut self.names, temp_dummy);
        self.names = Names::new(Some(Box::new(old_names)), None);
        self.convert_param_name_nodes(params)
    }

    pub fn push_function_context(
        &mut self,
        name: Name,
        params: ParamNameNodes,
    ) -> Result<(Name, ParamNameNodes), QErrorNode> {
        let temp_dummy = Names::new_root();
        let old_names = std::mem::replace(&mut self.names, temp_dummy);
        self.names = Names::new(Some(Box::new(old_names)), Some(name.bare_name().clone()));
        let converted_function_name = self.resolve_name_to_name(name);
        Ok((
            converted_function_name,
            self.convert_param_name_nodes(params)?,
        ))
    }

    pub fn pop_context(&mut self) {
        let temp_dummy = Names::new_root();
        let Names {
            parent,
            mut extended_names,
            ..
        } = std::mem::replace(&mut self.names, temp_dummy);
        self.names_without_dot
            .extend(extended_names.drain().map(|(k, _)| k));
        match parent {
            Some(boxed_parent) => {
                self.names = *boxed_parent;
            }
            _ => panic!("Stack underflow"),
        }
    }

    pub fn names_without_dot(mut self) -> HashSet<BareName> {
        self.names_without_dot
            .extend(self.names.extended_names.drain().map(|(k, _)| k));
        self.names_without_dot
    }

    pub fn on_expression(
        &mut self,
        expr_node: ExpressionNode,
        expr_context: ExprContext,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        expr_rules::on_expression(self, expr_node, expr_context)
    }

    pub fn on_opt_expression(
        &mut self,
        opt_expr_node: Option<ExpressionNode>,
        expr_context: ExprContext,
    ) -> Result<(Option<ExpressionNode>, Vec<QualifiedNameNode>), QErrorNode> {
        match opt_expr_node {
            Some(expr_node) => self
                .on_expression(expr_node, expr_context)
                .map(|(x, y)| (Some(x), y)),
            _ => Ok((None, vec![])),
        }
    }

    pub fn on_expressions(
        &mut self,
        expr_nodes: ExpressionNodes,
        expr_context: ExprContext,
    ) -> Result<(ExpressionNodes, Vec<QualifiedNameNode>), QErrorNode> {
        let mut implicit_vars: Vec<QualifiedNameNode> = vec![];
        let mut converted_expr_nodes: ExpressionNodes = vec![];
        for expr_node in expr_nodes {
            let (converted_expr_node, implicits) = self.on_expression(expr_node, expr_context)?;
            converted_expr_nodes.push(converted_expr_node);
            implicit_vars = union(implicit_vars, implicits);
        }
        Ok((converted_expr_nodes, implicit_vars))
    }

    pub fn on_assignment(
        &mut self,
        left_side: ExpressionNode,
        right_side: ExpressionNode,
    ) -> Result<(ExpressionNode, ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        assignment_pre_conversion_validation_rules::validate(self, &left_side)?;
        let (converted_right_side, right_side_implicit_vars) =
            self.on_expression(right_side, ExprContext::Default)?;
        let (converted_left_side, left_side_implicit_vars) =
            expr_rules::on_expression(self, left_side, ExprContext::Assignment)?;
        assignment_post_conversion_validation_rules::validate(
            self,
            &converted_left_side,
            &converted_right_side,
        )?;
        Ok((
            converted_left_side,
            converted_right_side,
            union(left_side_implicit_vars, right_side_implicit_vars),
        ))
    }

    pub fn on_dim(
        &mut self,
        dim_name_node: DimNameNode,
    ) -> Result<(DimNameNode, Vec<QualifiedNameNode>), QErrorNode> {
        dim_rules::on_dim(self, dim_name_node, false)
    }

    pub fn on_const(
        &mut self,
        left_side: NameNode,
        right_side: ExpressionNode,
    ) -> Result<(), QErrorNode> {
        const_rules::on_const(self, left_side, right_side)
    }

    fn convert_param_name_nodes(
        &mut self,
        params: ParamNameNodes,
    ) -> Result<ParamNameNodes, QErrorNode> {
        params
            .into_iter()
            .map(|x| self.convert_param_name_node(x))
            .collect()
    }

    fn convert_param_name_node(
        &mut self,
        param_name_node: ParamNameNode,
    ) -> Result<ParamNameNode, QErrorNode> {
        let Locatable {
            element:
                ParamName {
                    bare_name,
                    param_type,
                },
            pos,
        } = param_name_node;
        let dim_type = DimType::from(param_type);
        let dim_name_node: DimNameNode = DimNameBuilder::new()
            .bare_name(bare_name)
            .dim_type(dim_type)
            .build()
            .at(pos);
        let (converted_dim_name_node, implicits) = dim_rules::on_dim(self, dim_name_node, true)?;
        if implicits.is_empty() {
            let Locatable {
                element:
                    DimName {
                        bare_name,
                        dim_type,
                        ..
                    },
                pos,
            } = converted_dim_name_node;
            let param_type = ParamType::from(dim_type);
            let param_name = ParamName::new(bare_name, param_type);
            Ok(param_name.at(pos))
        } else {
            panic!("Should not have introduced implicit variables via parameter")
        }
    }

    fn function_qualifier(&self, bare_name: &BareName) -> Option<TypeQualifier> {
        match self.functions.get(bare_name) {
            Some(Locatable {
                element: (q, _), ..
            }) => Some(*q),
            _ => None,
        }
    }
}

enum RuleResult<I, O> {
    Success(O),
    Skip(I),
}

trait Rule<I, O, E> {
    fn eval(&self, ctx: &mut Context, input: I) -> Result<RuleResult<I, O>, E>;

    fn chain<X: Rule<I, O, E>>(self, other: X) -> Chain<Self, X>
    where
        Self: Sized,
    {
        Chain {
            left: self,
            right: other,
        }
    }

    fn chain_fn(
        self,
        f: fn(&mut Context, I) -> Result<RuleResult<I, O>, E>,
    ) -> Chain<Self, FnRule<I, O, E>>
    where
        Self: Sized,
    {
        Chain {
            left: self,
            right: FnRule::new(f),
        }
    }

    fn demand(&self, ctx: &mut Context, input: I) -> Result<O, E>
    where
        I: Debug,
    {
        match self.eval(ctx, input) {
            Ok(RuleResult::Success(o)) => Ok(o),
            Err(err) => Err(err),
            Ok(RuleResult::Skip(input)) => panic!("Could not process {:?}", input),
        }
    }
}

struct Chain<A, B> {
    pub left: A,
    pub right: B,
}

impl<A, B, I, O, E> Rule<I, O, E> for Chain<A, B>
where
    A: Rule<I, O, E>,
    B: Rule<I, O, E>,
{
    fn eval(&self, ctx: &mut Context, input: I) -> Result<RuleResult<I, O>, E> {
        match self.left.eval(ctx, input) {
            Ok(RuleResult::Success(s)) => Ok(RuleResult::Success(s)),
            Err(e) => Err(e),
            Ok(RuleResult::Skip(i)) => self.right.eval(ctx, i),
        }
    }
}

struct FnRule<I, O, E> {
    f: fn(&mut Context, I) -> Result<RuleResult<I, O>, E>,
}

impl<I, O, E> FnRule<I, O, E> {
    pub fn new(f: fn(&mut Context, I) -> Result<RuleResult<I, O>, E>) -> Self {
        Self { f }
    }
}

impl<I, O, E> Rule<I, O, E> for FnRule<I, O, E> {
    fn eval(&self, ctx: &mut Context, input: I) -> Result<RuleResult<I, O>, E> {
        (self.f)(ctx, input)
    }
}

pub mod expr_rules {
    use std::convert::TryFrom;

    use crate::built_ins::BuiltInFunction;
    use crate::common::{Location, QError, ToLocatableError};

    use super::*;

    type O = (ExpressionNode, Vec<QualifiedNameNode>);
    type R = std::result::Result<O, QErrorNode>;

    pub fn on_expression(
        ctx: &mut Context,
        expr_node: ExpressionNode,
        expr_context: ExprContext,
    ) -> std::result::Result<O, QErrorNode> {
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
                panic!("Parser is not supposed to produce any ArrayElement expressions, only FunctionCall")
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
                let unary_expr =
                    Expression::UnaryExpression(unary_operator, Box::new(converted_child));
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
            let (converted_left, left_implicit_vars) = ctx.on_expression(*left, expr_context)?;
            let (converted_right, right_implicit_vars) = ctx.on_expression(*right, expr_context)?;
            let new_expr = Expression::binary(converted_left, converted_right, binary_operator)?;
            Ok((
                new_expr.at(pos),
                union(left_implicit_vars, right_implicit_vars),
            ))
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

        fn validate(
            ctx: &Context,
            name: &Name,
            pos: Location,
        ) -> std::result::Result<(), QErrorNode> {
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
            ) -> std::result::Result<ExpressionNode, QErrorNode>;

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
            ) -> std::result::Result<ExpressionNode, QErrorNode> {
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
            ) -> std::result::Result<ExpressionNode, QErrorNode> {
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
            ) -> std::result::Result<ExpressionNode, QErrorNode> {
                let function_qualifier = self.function_qualifier.unwrap();
                let bare_name = name.bare_name();
                if name.is_bare_or_of_type(function_qualifier) {
                    if ctx.names.current_function_name.as_ref() == Some(bare_name) {
                        let converted_name = name.qualify(function_qualifier);
                        let expr_type = ExpressionType::BuiltIn(function_qualifier);
                        let expr = Expression::Variable(
                            converted_name,
                            VariableInfo::new_local(expr_type),
                        );
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
            ) -> std::result::Result<ExpressionNode, QErrorNode> {
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
            ) -> std::result::Result<ExpressionNode, QErrorNode> {
                let converted_name = name.qualify(self.function_qualifier.unwrap());
                Ok(Expression::FunctionCall(converted_name, vec![]).at(pos))
            }
        }
    }

    pub mod property_v2 {
        use super::*;
        use crate::linter::converter::context::expr_rules::variable_v2::{
            add_as_new_implicit_var, AssignToFunction, ExistingVar, VarAsUserDefinedFunctionCall,
            VarResolve,
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

                // TODO what about const with dot?

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
                Expression::Parenthesis(_) => {
                    todo!()
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

        fn resolve_function(
            ctx: &mut Context,
            name: Name,
            args: ExpressionNodes,
            pos: Location,
        ) -> R {
            // convert args
            let (converted_args, implicit_vars) =
                ctx.on_expressions(args, ExprContext::Parameter)?;
            // is it built-in function?
            let converted_expr =
                match Option::<BuiltInFunction>::try_from(&name).with_err_at(pos)? {
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
            ) -> std::result::Result<O, QErrorNode>;
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
            ) -> std::result::Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode>
            {
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
}

pub mod dim_rules {
    use std::convert::TryFrom;

    use crate::common::{QError, ToLocatableError};
    use crate::variant::MAX_INTEGER;

    use super::*;

    type I = DimNameNode;
    type O = (DimNameNode, Vec<QualifiedNameNode>);

    pub fn on_dim(
        ctx: &mut Context,
        dim_name_node: DimNameNode,
        is_param: bool,
    ) -> Result<(DimNameNode, Vec<QualifiedNameNode>), QErrorNode> {
        validate(ctx, &dim_name_node, is_param)?;
        new_var(ctx, dim_name_node)
    }

    fn validate(ctx: &Context, dim_name_node: &I, is_param: bool) -> Result<(), QErrorNode> {
        cannot_clash_with_subs::validate(ctx, dim_name_node)?;
        if is_param {
            cannot_clash_with_functions_param::validate(ctx, dim_name_node)?;
        } else {
            cannot_clash_with_functions::validate(ctx, dim_name_node)?;
        }
        cannot_clash_with_existing_names::validate(ctx, dim_name_node)?;
        user_defined_type_must_exist::validate(ctx, dim_name_node)?;
        shared_not_allowed_in_subprogram::validate(ctx, dim_name_node)
    }

    fn new_var(
        ctx: &mut Context,
        input: I,
    ) -> Result<(DimNameNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (converted_input, implicit_vars) = new_var_not_adding_to_context(ctx, input)?;
        // add to context
        let DimName {
            bare_name,
            dim_type,
            shared,
        } = converted_input.as_ref();
        let variable_context = VariableInfo {
            expression_type: dim_type.expression_type(),
            shared: *shared,
        };
        if converted_input.is_extended() {
            ctx.names
                .insert_extended(bare_name.clone(), variable_context);
        } else {
            let q = TypeQualifier::try_from(&converted_input)?;
            ctx.names
                .insert_compact(bare_name.clone(), q, variable_context);
        }
        Ok((converted_input, implicit_vars))
    }

    fn new_var_not_adding_to_context(ctx: &mut Context, input: I) -> Result<O, QErrorNode> {
        let Locatable {
            element:
                DimName {
                    bare_name,
                    dim_type,
                    shared,
                },
            pos,
        } = input;
        match dim_type {
            DimType::Bare => {
                let qualifier = ctx.resolve(&bare_name);
                let dim_type = DimType::BuiltIn(qualifier, BuiltInStyle::Compact);
                let converted_dim_name = DimName::new(bare_name, dim_type, shared);
                Ok((converted_dim_name.at(pos), vec![]))
            }
            DimType::BuiltIn(q, built_in_style) => {
                let result =
                    DimName::new(bare_name, DimType::BuiltIn(q, built_in_style), shared).at(pos);
                Ok((result, vec![]))
            }
            DimType::FixedLengthString(len_expr, _) => {
                let v = ctx.names.resolve_const_value_node(&len_expr)?;
                if let Variant::VInteger(len) = v {
                    if len > 1 && len < MAX_INTEGER {
                        let result = DimName::new(
                            bare_name,
                            DimType::FixedLengthString(
                                Expression::IntegerLiteral(len).at(&len_expr),
                                len as u16,
                            ),
                            shared,
                        )
                        .at(pos);
                        Ok((result, vec![]))
                    } else {
                        Err(QError::OutOfStringSpace).with_err_at(&len_expr)
                    }
                } else {
                    Err(QError::TypeMismatch).with_err_at(&len_expr)
                }
            }
            DimType::UserDefined(user_defined_type_name_node) => {
                let result = DimName::new(
                    bare_name,
                    DimType::UserDefined(user_defined_type_name_node),
                    shared,
                )
                .at(pos);
                Ok((result, vec![]))
            }
            DimType::Array(dimensions, boxed_element_type) => {
                // dimensions
                let mut converted_dimensions: ArrayDimensions = vec![];
                let mut implicit_vars: Vec<QualifiedNameNode> = vec![];
                for dimension in dimensions {
                    let ArrayDimension {
                        lbound: opt_lbound,
                        ubound,
                    } = dimension;
                    let (converted_lbound, implicit_vars_lbound) = match opt_lbound {
                        Some(lbound) => ctx
                            .on_expression(lbound, ExprContext::Default)
                            .map(|(x, y)| (Some(x), y))?,
                        _ => (None, vec![]),
                    };
                    let (converted_ubound, implicit_vars_ubound) =
                        ctx.on_expression(ubound, ExprContext::Default)?;
                    converted_dimensions.push(ArrayDimension {
                        lbound: converted_lbound,
                        ubound: converted_ubound,
                    });
                    implicit_vars = union(implicit_vars, implicit_vars_lbound);
                    implicit_vars = union(implicit_vars, implicit_vars_ubound);
                }
                // dim_type
                let element_dim_type = *boxed_element_type;
                let element_dim_name =
                    DimName::new(bare_name.clone(), element_dim_type, shared).at(pos);
                let (
                    Locatable {
                        element: DimName { dim_type, .. },
                        ..
                    },
                    implicits,
                ) = new_var_not_adding_to_context(ctx, element_dim_name)?;
                implicit_vars = union(implicit_vars, implicits);
                let array_dim_type = DimType::Array(converted_dimensions, Box::new(dim_type));
                Ok((
                    DimName::new(bare_name, array_dim_type, shared).at(pos),
                    implicit_vars,
                ))
            }
        }
    }

    pub mod cannot_clash_with_subs {
        use super::*;

        pub fn validate(ctx: &Context, dim_name_node: &DimNameNode) -> Result<(), QErrorNode> {
            if ctx.subs.contains_key(dim_name_node.bare_name()) {
                Err(QError::DuplicateDefinition).with_err_at(dim_name_node)
            } else {
                Ok(())
            }
        }
    }

    pub mod cannot_clash_with_functions {
        use super::*;

        pub fn validate(ctx: &Context, dim_name_node: &DimNameNode) -> Result<(), QErrorNode> {
            if ctx.functions.contains_key(dim_name_node.bare_name()) {
                Err(QError::DuplicateDefinition).with_err_at(dim_name_node)
            } else {
                Ok(())
            }
        }
    }

    pub mod cannot_clash_with_functions_param {
        use super::*;

        pub fn validate(ctx: &Context, dim_name_node: &DimNameNode) -> Result<(), QErrorNode> {
            match ctx.function_qualifier(dim_name_node.bare_name()) {
                Some(func_qualifier) => {
                    if dim_name_node.is_extended() {
                        Err(QError::DuplicateDefinition).with_err_at(dim_name_node)
                    } else {
                        let q = ctx.resolve_name_ref_to_qualifier(dim_name_node);
                        if q == func_qualifier {
                            // for some reason you can have a FUNCTION Add(Add)
                            Ok(())
                        } else {
                            Err(QError::DuplicateDefinition).with_err_at(dim_name_node)
                        }
                    }
                }
                _ => Ok(()),
            }
        }
    }

    pub mod cannot_clash_with_existing_names {
        use super::*;

        pub fn validate(ctx: &Context, dim_name_node: &DimNameNode) -> Result<(), QErrorNode> {
            if dim_name_node.is_extended() {
                if ctx
                    .names
                    .contains_local_var_or_local_const(dim_name_node.bare_name())
                    || ctx
                        .names
                        .get_extended_var_recursively(dim_name_node.bare_name())
                        .is_some()
                {
                    Err(QError::DuplicateDefinition).with_err_at(dim_name_node)
                } else {
                    Ok(())
                }
            } else {
                let qualifier = ctx.resolve_name_ref_to_qualifier(dim_name_node);
                if ctx
                    .names
                    .can_accept_compact(dim_name_node.bare_name(), qualifier)
                {
                    Ok(())
                } else {
                    Err(QError::DuplicateDefinition).with_err_at(dim_name_node)
                }
            }
        }
    }

    pub mod user_defined_type_must_exist {
        use super::*;

        pub fn validate(ctx: &Context, dim_name_node: &DimNameNode) -> Result<(), QErrorNode> {
            if let Some(user_defined_type_name_node) = dim_name_node.is_user_defined() {
                if ctx
                    .user_defined_types
                    .contains_key(user_defined_type_name_node.as_ref())
                {
                    Ok(())
                } else {
                    Err(QError::TypeNotDefined).with_err_at(user_defined_type_name_node)
                }
            } else {
                Ok(())
            }
        }
    }

    pub mod shared_not_allowed_in_subprogram {
        use super::*;

        pub fn validate(ctx: &Context, dim_name_node: &DimNameNode) -> Result<(), QErrorNode> {
            if ctx.names.parent.is_some() && dim_name_node.shared {
                Err(QError::syntax_error("SHARED not allowed in subprogram"))
                    .with_err_at(dim_name_node)
            } else {
                Ok(())
            }
        }
    }
}

pub mod const_rules {
    use std::convert::TryFrom;

    use crate::common::{QError, ToLocatableError};

    use super::*;

    type I = (NameNode, ExpressionNode);
    type O = ();
    type ConstResult = RuleResult<I, O>;
    type Result = std::result::Result<ConstResult, QErrorNode>;

    pub fn on_const(
        ctx: &mut Context,
        left_side: NameNode,
        right_side: ExpressionNode,
    ) -> std::result::Result<O, QErrorNode> {
        if ctx
            .names
            .get_extended_var_recursively(left_side.bare_name())
            .is_some()
        {
            return Err(QError::DuplicateDefinition).with_err_at(&left_side);
        }
        let rule = FnRule::new(const_cannot_clash_with_existing_names).chain_fn(new_const);
        rule.demand(ctx, (left_side, right_side))
    }

    fn const_cannot_clash_with_existing_names(ctx: &mut Context, input: I) -> Result {
        let (
            Locatable {
                element: const_name,
                pos: const_name_pos,
            },
            right,
        ) = input;
        if ctx
            .names
            .contains_local_var_or_local_const(const_name.bare_name())
            || ctx.subs.contains_key(const_name.bare_name())
            || ctx.functions.contains_key(const_name.bare_name())
        {
            Err(QError::DuplicateDefinition).with_err_at(const_name_pos)
        } else {
            Ok(RuleResult::Skip((const_name.at(const_name_pos), right)))
        }
    }

    fn new_const(ctx: &mut Context, input: I) -> Result {
        let (
            Locatable {
                element: const_name,
                ..
            },
            right,
        ) = input;
        let value_before_casting = ctx.names.resolve_const_value_node(&right)?;
        let value_qualifier = TypeQualifier::try_from(&value_before_casting).with_err_at(&right)?;
        let final_value = if const_name.is_bare_or_of_type(value_qualifier) {
            value_before_casting
        } else {
            value_before_casting
                .cast(const_name.qualifier().unwrap())
                .with_err_at(&right)?
        };
        ctx.names
            .insert_const(const_name.bare_name().clone(), final_value.clone());
        Ok(RuleResult::Success(()))
    }
}

pub mod assignment_pre_conversion_validation_rules {
    use crate::common::{QError, ToLocatableError};

    use super::*;

    type I<'a> = &'a ExpressionNode;
    type ValidationResult<'a> = RuleResult<I<'a>, ()>;
    type Result<'a> = std::result::Result<ValidationResult<'a>, QErrorNode>;

    pub fn validate(
        ctx: &mut Context,
        left_side: &ExpressionNode,
    ) -> std::result::Result<(), QErrorNode> {
        let rule = FnRule::new(cannot_assign_to_const).chain_fn(success);
        rule.demand(ctx, left_side)
    }

    fn cannot_assign_to_const<'a>(ctx: &mut Context, input: I<'a>) -> Result<'a> {
        if let Locatable {
            element: Expression::Variable(var_name, _),
            ..
        } = input
        {
            if ctx.names.contains_const_recursively(var_name.bare_name()) {
                Err(QError::DuplicateDefinition).with_err_at(input)
            } else {
                Ok(RuleResult::Skip(input))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn success<'a>(_ctx: &mut Context, _input: I<'a>) -> Result<'a> {
        Ok(RuleResult::Success(()))
    }
}

pub mod assignment_post_conversion_validation_rules {
    use crate::common::{CanCastTo, QError, ToLocatableError};

    use super::*;

    type I<'a> = (&'a ExpressionNode, &'a ExpressionNode);
    type ValidationResult<'a> = RuleResult<I<'a>, ()>;
    type Result<'a> = std::result::Result<ValidationResult<'a>, QErrorNode>;

    pub fn validate(
        ctx: &mut Context,
        left_side: &ExpressionNode,
        right_side: &ExpressionNode,
    ) -> std::result::Result<(), QErrorNode> {
        let rule = FnRule::new(can_cast_right_to_left).chain_fn(success);
        rule.demand(ctx, (left_side, right_side))
    }

    fn can_cast_right_to_left<'a>(_ctx: &mut Context, input: I<'a>) -> Result<'a> {
        let (left, right) = input;
        if right.can_cast_to(left) {
            Ok(RuleResult::Skip((left, right)))
        } else {
            Err(QError::TypeMismatch).with_err_at(right)
        }
    }

    fn success<'a>(_ctx: &mut Context, _input: I<'a>) -> Result<'a> {
        Ok(RuleResult::Success(()))
    }
}

fn union(
    mut left: Vec<QualifiedNameNode>,
    mut right: Vec<QualifiedNameNode>,
) -> Vec<QualifiedNameNode> {
    left.append(&mut right);
    left
}
