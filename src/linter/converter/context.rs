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

    pub fn contains_any(&self, bare_name: &BareName) -> bool {
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

    pub fn contains_compact(
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

    pub fn contains_extended(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        self.extended_names.get(bare_name)
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
    use crate::common::{QError, ToLocatableError};

    use super::*;

    type I = ExpressionNode;
    type O = (ExpressionNode, Vec<QualifiedNameNode>);
    type ExprResult = RuleResult<I, O>;
    type Result = std::result::Result<ExprResult, QErrorNode>;

    pub fn on_expression(
        ctx: &mut Context,
        expr_node: ExpressionNode,
        expr_context: ExprContext,
    ) -> std::result::Result<O, QErrorNode> {
        let conversion_rules = FnRule::new(literals)
            .chain_fn(variable_name_clashes_with_sub)
            .chain_fn(variable_existing_extended_var)
            .chain_fn(variable_existing_const)
            .chain_fn(function_call_existing_extended_array_with_parenthesis)
            .chain_fn(function_call_existing_compact_array_with_parenthesis)
            .chain_fn(variable_or_property_existing_compact_name)
            .chain_fn(variable_existing_global_shared_var_compact)
            .chain_fn(if expr_context != ExprContext::Default {
                variable_or_property_assign_to_function
            } else {
                variable_or_property_as_function_call
            })
            .chain_fn(unary_expr)
            .chain_fn(binary_expr)
            .chain_fn(function_call_must_have_args)
            .chain_fn(function_call)
            .chain_fn(property_of_existing_var)
            .chain_fn(variable_existing_parent_const)
            .chain_fn(property_fold_property_into_implicit_var)
            .chain_fn(variable_implicit_var)
            .chain_fn(parenthesis);
        conversion_rules.demand(ctx, expr_node)
    }

    fn variable_or_property_assign_to_function(ctx: &mut Context, input: I) -> Result {
        let Locatable { element: expr, pos } = input;
        match expr.fold_name() {
            Some(name) => {
                let bare_name: &BareName = name.bare_name();
                // if a function of this name exists...
                match ctx.function_qualifier(bare_name) {
                    Some(function_qualifier) => {
                        // and the name qualifier matches...
                        if name.is_bare_or_of_type(function_qualifier) {
                            // and (since we're assigning to it) it's the current function
                            if ctx.names.current_function_name.as_ref() == Some(bare_name) {
                                let converted_name = name.qualify(function_qualifier);
                                let expr_type = ExpressionType::BuiltIn(function_qualifier);
                                let expr = Expression::Variable(
                                    converted_name,
                                    VariableInfo::new_local(expr_type),
                                );
                                Ok(RuleResult::Success((expr.at(pos), vec![])))
                            } else {
                                Err(QError::DuplicateDefinition).with_err_at(pos)
                            }
                        } else {
                            Err(QError::DuplicateDefinition).with_err_at(pos)
                        }
                    }
                    _ => Ok(RuleResult::Skip(expr.at(pos))),
                }
            }
            _ => Ok(RuleResult::Skip(expr.at(pos))),
        }
    }

    fn variable_name_clashes_with_sub(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, _),
            pos,
        } = &input
        {
            if ctx.subs.contains_key(name.bare_name()) {
                Err(QError::DuplicateDefinition).with_err_at(*pos)
            } else {
                Ok(RuleResult::Skip(input))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn variable_existing_extended_var(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, expr_type),
            pos,
        } = input
        {
            let bare_name = name.bare_name();
            if let Some(VariableInfo {
                expression_type: expr_type,
                ..
            }) = ctx.names.contains_extended(bare_name)
            {
                let converted_name = expr_type.qualify_name(name).with_err_at(pos)?;
                Ok(RuleResult::Success((
                    Expression::Variable(
                        converted_name,
                        VariableInfo::new_local(expr_type.clone()),
                    )
                    .at(pos),
                    vec![],
                )))
            } else {
                Ok(RuleResult::Skip(
                    Expression::Variable(name, expr_type).at(pos),
                ))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn variable_existing_const(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, expr_type),
            pos,
        } = input
        {
            if let Some(v) = ctx.names.get_const_value_no_recursion(name.bare_name()) {
                let q: TypeQualifier = TypeQualifier::try_from(v).with_err_at(pos)?;
                if name.is_bare_or_of_type(q) {
                    // resolve to literal expr
                    let expr = Expression::try_from(v.clone()).with_err_at(pos)?;
                    Ok(RuleResult::Success((expr.at(pos), vec![])))
                } else {
                    Err(QError::DuplicateDefinition).with_err_at(pos)
                }
            } else {
                Ok(RuleResult::Skip(
                    Expression::Variable(name, expr_type).at(pos),
                ))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn variable_existing_parent_const(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, expr_type),
            pos,
        } = input
        {
            if let Some(v) = ctx.names.get_const_value_recursively(name.bare_name()) {
                let q: TypeQualifier = TypeQualifier::try_from(v).with_err_at(pos)?;
                if name.is_bare_or_of_type(q) {
                    // resolve to literal expr
                    let expr = Expression::try_from(v.clone()).with_err_at(pos)?;
                    Ok(RuleResult::Success((expr.at(pos), vec![])))
                } else {
                    Err(QError::DuplicateDefinition).with_err_at(pos)
                }
            } else {
                Ok(RuleResult::Skip(
                    Expression::Variable(name, expr_type).at(pos),
                ))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn variable_or_property_existing_compact_name(ctx: &mut Context, input: I) -> Result {
        let Locatable { element: expr, pos } = input;
        match expr.fold_name() {
            Some(name) => {
                let bare_name: &BareName = name.bare_name();
                let q: TypeQualifier = ctx.resolve_name_to_qualifier(&name);
                if let Some(VariableInfo {
                    expression_type: expr_type,
                    ..
                }) = ctx.names.contains_compact(bare_name, q)
                {
                    let converted_name = name.qualify(q);
                    let expr = Expression::Variable(
                        converted_name,
                        VariableInfo::new_local(expr_type.clone()),
                    );
                    Ok(RuleResult::Success((expr.at(pos), vec![])))
                } else {
                    Ok(RuleResult::Skip(expr.at(pos)))
                }
            }
            _ => Ok(RuleResult::Skip(expr.at(pos))),
        }
    }

    fn variable_existing_global_shared_var_compact(ctx: &mut Context, input: I) -> Result {
        match &ctx.names.parent {
            Some(parent_names) => {
                let Locatable { element, pos } = input;
                match element.fold_name() {
                    Some(name) => {
                        let bare_name: &BareName = name.bare_name();
                        let q: TypeQualifier = ctx.resolve_name_to_qualifier(&name);
                        match parent_names.contains_compact(bare_name, q) {
                            Some(var_context) => {
                                if var_context.shared {
                                    let converted_name = name.qualify(q);
                                    let expr =
                                        Expression::Variable(converted_name, var_context.clone());
                                    Ok(RuleResult::Success((expr.at(pos), vec![])))
                                } else {
                                    Ok(RuleResult::Skip(element.at(pos)))
                                }
                            }
                            _ => Ok(RuleResult::Skip(element.at(pos))),
                        }
                    }
                    _ => Ok(RuleResult::Skip(element.at(pos))),
                }
            }
            _ => Ok(RuleResult::Skip(input)),
        }
    }

    fn literals(_ctx: &mut Context, input: I) -> Result {
        let Locatable { element, pos } = input;
        match element {
            Expression::SingleLiteral(_)
            | Expression::DoubleLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::IntegerLiteral(_)
            | Expression::LongLiteral(_) => Ok(RuleResult::Success((element.at(pos), vec![]))),
            _ => Ok(RuleResult::Skip(element.at(pos))),
        }
    }

    fn binary_expr(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::BinaryExpression(op, left, right, _),
            pos,
        } = input
        {
            let (converted_left, left_implicit_vars) =
                ctx.on_expression(*left, ExprContext::Default)?;
            let (converted_right, right_implicit_vars) =
                ctx.on_expression(*right, ExprContext::Default)?;
            let new_expr = Expression::binary(converted_left, converted_right, op)?;
            Ok(RuleResult::Success((
                new_expr.at(pos),
                union(left_implicit_vars, right_implicit_vars),
            )))
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn unary_expr(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::UnaryExpression(op, child),
            pos,
        } = input
        {
            let (converted_child, implicit_vars) =
                ctx.on_expression(*child, ExprContext::Default)?;
            if op.applies_to(&converted_child.expression_type()) {
                let new_expr = Expression::UnaryExpression(op, Box::new(converted_child));
                Ok(RuleResult::Success((new_expr.at(pos), implicit_vars)))
            } else {
                Err(QError::TypeMismatch).with_err_at(&converted_child)
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn variable_implicit_var(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, _),
            pos,
        } = input
        {
            let resolved_name = ctx.resolve_name_to_name(name);
            let q_name = resolved_name.clone().demand_qualified();
            let qualifier = q_name.qualifier;
            let implicit_vars = vec![q_name.at(pos)];
            let expr_type = ExpressionType::BuiltIn(qualifier);
            ctx.names.insert_compact(
                resolved_name.bare_name().clone(),
                qualifier,
                VariableInfo::new_local(expr_type.clone()),
            );
            let resoled_expr =
                Expression::Variable(resolved_name, VariableInfo::new_local(expr_type)).at(pos);
            Ok(RuleResult::Success((resoled_expr, implicit_vars)))
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn property_of_existing_var(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Property(left, right, old_expr_type),
            pos,
        } = input
        {
            // Find the most left name. If it is an existing variable, expect all to be resolved.
            // If it is not an existing variable, skip and allow the fold rule to follow.
            if let Some(left_most_name) = left.left_most_name() {
                if let Some(VariableInfo {
                    expression_type: ExpressionType::UserDefined(_),
                    ..
                }) = ctx.names.contains_extended(left_most_name.bare_name())
                {
                    let (
                        Locatable {
                            element: converted_left,
                            ..
                        },
                        implicit_left,
                    ) = ctx.on_expression((*left).at(pos), ExprContext::Default)?;
                    let converted_left_expr_type = converted_left.expression_type();
                    let converted_left_element_type = converted_left_expr_type.to_element_type();
                    if let ExpressionType::UserDefined(user_defined_type_name) =
                        converted_left_element_type
                    {
                        if let Some(user_defined_type) =
                            ctx.user_defined_types.get(user_defined_type_name)
                        {
                            if let Some(element_type) =
                                user_defined_type.find_element(right.bare_name())
                            {
                                if element_type.can_be_referenced_by_property_name(&right) {
                                    Ok(RuleResult::Success((
                                        Expression::Property(
                                            Box::new(converted_left),
                                            right.un_qualify(),
                                            element_type.expression_type(),
                                        )
                                        .at(pos),
                                        implicit_left,
                                    )))
                                } else {
                                    // using wrong qualifier
                                    Err(QError::TypeMismatch).with_err_at(pos)
                                }
                            } else {
                                Err(QError::ElementNotDefined).with_err_at(pos)
                            }
                        } else {
                            Err(QError::TypeNotDefined).with_err_at(pos)
                        }
                    } else {
                        Err(QError::TypeMismatch).with_err_at(pos)
                    }
                } else {
                    Ok(RuleResult::Skip(
                        Expression::Property(left, right, old_expr_type).at(pos),
                    ))
                }
            } else {
                Ok(RuleResult::Skip(
                    Expression::Property(left, right, old_expr_type).at(pos),
                ))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn property_fold_property_into_implicit_var(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Property(left, right, _),
            pos,
        } = input
        {
            match left.fold_name() {
                Some(folded_left_name) => match folded_left_name.try_concat_name(right) {
                    Some(folded_name) => variable_implicit_var(
                        ctx,
                        Expression::Variable(
                            folded_name,
                            VariableInfo::new_local(ExpressionType::Unresolved),
                        )
                        .at(pos),
                    ),
                    _ => Err(QError::ElementNotDefined).with_err_at(pos),
                },
                _ => Err(QError::ElementNotDefined).with_err_at(pos),
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn function_call_existing_extended_array_with_parenthesis(
        ctx: &mut Context,
        input: I,
    ) -> Result {
        if let Locatable {
            element: Expression::FunctionCall(name, args),
            pos,
        } = input
        {
            let bare_name = name.bare_name();
            if let Some(VariableInfo {
                expression_type: ExpressionType::Array(boxed_element_type),
                shared,
            }) = ctx.names.contains_extended(bare_name)
            {
                // clone element type early in order to be able to use ctx as mutable later
                let element_type = boxed_element_type.as_ref().clone();
                let shared = *shared;
                // convert args
                let (converted_args, implicit_vars) =
                    ctx.on_expressions(args, ExprContext::Default)?;
                // convert name
                let converted_name = element_type.qualify_name(name).with_err_at(pos)?;
                // create result
                let result_expr = Expression::ArrayElement(
                    converted_name,
                    converted_args,
                    VariableInfo {
                        expression_type: element_type,
                        shared,
                    },
                );
                Ok(RuleResult::Success((result_expr.at(pos), implicit_vars)))
            } else {
                Ok(RuleResult::Skip(
                    Expression::FunctionCall(name, args).at(pos),
                ))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn function_call_existing_compact_array_with_parenthesis(
        ctx: &mut Context,
        input: I,
    ) -> Result {
        if let Locatable {
            element: Expression::FunctionCall(name, args),
            pos,
        } = input
        {
            let bare_name: &BareName = name.bare_name();
            let qualifier: TypeQualifier = ctx.resolve_name_to_qualifier(&name);
            if let Some(VariableInfo {
                expression_type: ExpressionType::Array(boxed_element_type),
                shared,
            }) = ctx.names.contains_compact(bare_name, qualifier)
            {
                // clone element type early in order to be able to use ctx as mutable later
                let element_type = boxed_element_type.as_ref().clone();
                let shared = *shared;
                // convert args
                let (converted_args, implicit_vars) =
                    ctx.on_expressions(args, ExprContext::Default)?;
                // convert name
                let converted_name = name.qualify(qualifier);
                // create result
                let result_expr = Expression::ArrayElement(
                    converted_name,
                    converted_args,
                    VariableInfo {
                        expression_type: element_type,
                        shared,
                    },
                );
                Ok(RuleResult::Success((result_expr.at(pos), implicit_vars)))
            } else {
                Ok(RuleResult::Skip(
                    Expression::FunctionCall(name, args).at(pos),
                ))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn function_call_must_have_args(_ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::FunctionCall(_, args),
            ..
        } = &input
        {
            if args.is_empty() {
                Err(QError::syntax_error(
                    "Cannot have function call without arguments",
                ))
                .with_err_at(&input)
            } else {
                Ok(RuleResult::Skip(input))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn function_call(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::FunctionCall(name, args),
            pos,
        } = input
        {
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
            Ok(RuleResult::Success((converted_expr.at(pos), implicit_vars)))
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn variable_or_property_as_function_call(ctx: &mut Context, input: I) -> Result {
        let Locatable { element: expr, pos } = input;
        match expr.fold_name() {
            Some(name) => {
                // is it built-in function?
                match Option::<BuiltInFunction>::try_from(&name).with_err_at(pos)? {
                    Some(built_in_function) => Ok(RuleResult::Success((
                        Expression::BuiltInFunctionCall(built_in_function, vec![]).at(pos),
                        vec![],
                    ))),
                    _ => match ctx.function_qualifier(name.bare_name()) {
                        Some(q) => {
                            let converted_name = name.qualify(q);
                            Ok(RuleResult::Success((
                                Expression::FunctionCall(converted_name, vec![]).at(pos),
                                vec![],
                            )))
                        }
                        _ => Ok(RuleResult::Skip(expr.at(pos))),
                    },
                }
            }
            _ => Ok(RuleResult::Skip(expr.at(pos))),
        }
    }

    fn parenthesis(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Parenthesis(child),
            pos,
        } = input
        {
            let (converted_child, implicit_vars) =
                ctx.on_expression(*child, ExprContext::Default)?;
            let converted_expr = Expression::Parenthesis(Box::new(converted_child));
            Ok(RuleResult::Success((converted_expr.at(pos), implicit_vars)))
        } else {
            Ok(RuleResult::Skip(input))
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
                if ctx.names.contains_any(dim_name_node.bare_name()) {
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
        if ctx.names.contains_any(const_name.bare_name())
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
