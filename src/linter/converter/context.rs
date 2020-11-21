use crate::common::{AtLocation, CaseInsensitiveString, Locatable, QErrorNode};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::{
    BareName, DimNameNode, Expression, ExpressionNode, ExpressionType, FunctionMap, Name, NameNode,
    ParamNameNodes, QualifiedName, QualifiedNameNode, StatementNode, SubMap, TypeQualifier,
    UserDefinedTypes,
};
use crate::variant::Variant;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;

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
}

struct Names {
    names: HashMap<BareName, ExpressionType>,
    constants: HashMap<BareName, Variant>,
    parent: Option<Box<Names>>,
}

impl Names {
    pub fn new(parent: Option<Box<Self>>) -> Self {
        Self {
            names: HashMap::new(),
            constants: HashMap::new(),
            parent,
        }
    }

    pub fn contains_built_in_extended(&self, name: &Name) -> Option<TypeQualifier> {
        let bare_name: &BareName = name.as_ref();
        match self.names.get(bare_name) {
            Some(ExpressionType::BuiltIn(q)) => {
                if name.is_bare_or_of_type(*q) {
                    Some(*q)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn contains_const(&self, bare_name: &BareName) -> bool {
        self.constants.contains_key(bare_name)
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
            names: Names::new(None),
        }
    }

    pub fn push_sub_context(
        &mut self,
        params: ParamNameNodes,
    ) -> Result<ParamNameNodes, QErrorNode> {
        let temp_dummy = Names::new(None);
        let old_names = std::mem::replace(&mut self.names, temp_dummy);
        self.names = Names::new(Some(Box::new(old_names)));
        Ok(params)
    }

    pub fn push_function_context(
        &mut self,
        name: Name,
        params: ParamNameNodes,
    ) -> Result<(Name, ParamNameNodes), QErrorNode> {
        let temp_dummy = Names::new(None);
        let old_names = std::mem::replace(&mut self.names, temp_dummy);
        self.names = Names::new(Some(Box::new(old_names)));
        Ok((name, params))
    }

    pub fn pop_context(&mut self) {
        let temp_dummy = Names::new(None);
        let old_names = std::mem::replace(&mut self.names, temp_dummy);
        match old_names.parent {
            Some(boxed_parent) => {
                self.names = *boxed_parent;
            }
            _ => panic!("Stack underflow"),
        }
    }

    pub fn names_without_dot(self) -> HashSet<BareName> {
        HashSet::new()
    }

    pub fn on_expression(
        &mut self,
        expr_node: ExpressionNode,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        expr_rules::on_expression(self, expr_node)
    }

    pub fn on_assignment(
        &mut self,
        left_side: ExpressionNode,
        right_side: ExpressionNode,
    ) -> Result<(ExpressionNode, ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        assignment_pre_conversion_validation_rules::validate(self, &left_side)?;
        let (converted_right_side, mut right_side_implicit_vars) =
            self.on_expression(right_side)?;
        let (converted_left_side, mut left_side_implicit_vars) = self.on_expression(left_side)?;
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
        dim_rules::on_dim(self, dim_name_node)
    }

    pub fn on_const(
        &mut self,
        left_side: NameNode,
        right_side: ExpressionNode,
    ) -> Result<(NameNode, ExpressionNode, Variant), QErrorNode> {
        const_rules::on_const(self, left_side, right_side)
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
    use super::*;
    use crate::common::{QError, ToLocatableError};

    type I = ExpressionNode;
    type O = (ExpressionNode, Vec<QualifiedNameNode>);
    type ExprResult = RuleResult<I, O>;
    type Result = std::result::Result<ExprResult, QErrorNode>;

    pub fn on_expression(
        ctx: &mut Context,
        expr_node: ExpressionNode,
    ) -> std::result::Result<O, QErrorNode> {
        let conversion_rules = FnRule::new(literals)
            .chain_fn(name_clashes_with_sub)
            .chain_fn(existing_extended_var)
            .chain_fn(binary)
            .chain_fn(implicit_var)
            .chain_fn(fold_property_into_implicit_var);
        conversion_rules.demand(ctx, expr_node)
    }

    fn name_clashes_with_sub(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, expr_type),
            pos,
        } = &input
        {
            if ctx.subs.contains_key(name.as_ref()) {
                Err(QError::DuplicateDefinition).with_err_at(*pos)
            } else {
                Ok(RuleResult::Skip(input))
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn existing_extended_var(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, expr_type),
            pos,
        } = input
        {
            if let Some(q) = ctx.names.contains_built_in_extended(&name) {
                Ok(RuleResult::Success((
                    Expression::Variable(name.qualify(q), ExpressionType::BuiltIn(q)).at(pos),
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

    fn binary(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::BinaryExpression(op, left, right, _),
            pos,
        } = input
        {
            let (converted_left, mut left_implicit_vars) = ctx.on_expression(*left)?;
            let (converted_right, mut right_implicit_vars) = ctx.on_expression(*right)?;
            let new_expr = Expression::binary(converted_left, converted_right, op)?;
            Ok(RuleResult::Success((
                new_expr.at(pos),
                union(left_implicit_vars, right_implicit_vars),
            )))
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn implicit_var(ctx: &mut Context, input: I) -> Result {
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
            let resoled_expr = Expression::Variable(resolved_name, expr_type).at(pos);
            Ok(RuleResult::Success((resoled_expr, implicit_vars)))
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn fold_property_into_implicit_var(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Property(left, right, _),
            pos,
        } = input
        {
            match left.fold_name() {
                Some(folded_left_name) => match folded_left_name.try_concat_name(right) {
                    Some(folded_name) => implicit_var(
                        ctx,
                        Expression::Variable(folded_name, ExpressionType::Unresolved).at(pos),
                    ),
                    _ => Err(QError::ElementNotDefined).with_err_at(pos),
                },
                _ => Err(QError::ElementNotDefined).with_err_at(pos),
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }
}

pub mod dim_rules {
    use super::*;
    use crate::parser::{BuiltInStyle, DimName, DimNameTrait, DimType};

    type I = DimNameNode;
    type O = (DimNameNode, Vec<QualifiedNameNode>);
    type DimResult = RuleResult<I, O>;
    type Result = std::result::Result<DimResult, QErrorNode>;

    pub fn on_dim(
        ctx: &mut Context,
        dim_name_node: DimNameNode,
    ) -> std::result::Result<(DimNameNode, Vec<QualifiedNameNode>), QErrorNode> {
        let rule = FnRule::new(new_bare_var).chain_fn(new_built_in_extended_var);
        rule.demand(ctx, dim_name_node)
    }

    fn new_bare_var(ctx: &mut Context, input: I) -> Result {
        if input.is_bare() {
            let Locatable {
                element: dim_name,
                pos,
            } = input;
            let DimName { bare_name, .. } = dim_name;
            let qualifier = ctx.resolve(&bare_name);
            let dim_type = DimType::BuiltIn(qualifier, BuiltInStyle::Compact);
            let converted_dim_name = DimName::new(bare_name, dim_type);
            Ok(RuleResult::Success((converted_dim_name.at(pos), vec![])))
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn new_built_in_extended_var(ctx: &mut Context, input: I) -> Result {
        if let Some(q) = input.is_built_in_extended() {
            let bare_name: &BareName = input.as_ref();
            ctx.names
                .names
                .insert(bare_name.clone(), ExpressionType::BuiltIn(q));
            Ok(RuleResult::Success((input, vec![])))
        } else {
            Ok(RuleResult::Skip(input))
        }
    }
}

pub mod const_rules {
    use super::*;
    use crate::common::{QError, ToLocatableError};
    use std::convert::TryFrom;

    type I = (NameNode, ExpressionNode);
    type O = (NameNode, ExpressionNode, Variant);
    type ConstResult = RuleResult<I, O>;
    type Result = std::result::Result<ConstResult, QErrorNode>;

    pub fn on_const(
        ctx: &mut Context,
        left_side: NameNode,
        right_side: ExpressionNode,
    ) -> std::result::Result<O, QErrorNode> {
        let rule = FnRule::new(new_const);
        rule.demand(ctx, (left_side, right_side))
    }

    fn new_const(ctx: &mut Context, input: I) -> Result {
        let (
            Locatable {
                element: const_name,
                pos: const_post,
            },
            right,
        ) = input;
        let v = ctx.names.resolve_const_value_node(&right)?;
        let q = TypeQualifier::try_from(&v).with_err_at(&right)?;
        if const_name.is_bare_or_of_type(q) {
            ctx.names.constants.insert(const_name.as_ref().clone(), v.clone());
            let converted_name = const_name.qualify(q).at(const_post);
            let converted_expr = Expression::try_from(v.clone()).with_err_at(&right)?;
            let res = (converted_name, converted_expr.at(&right), v);
            Ok(RuleResult::Success(res))
        } else {
            Err(QError::TypeMismatch).with_err_at(&right)
        }
    }
}

pub mod assignment_pre_conversion_validation_rules {
    use super::*;
    use crate::common::{CanCastTo, QError, ToLocatableError};

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
        if let Locatable { element: Expression::Variable(var_name, _), pos } = input {
            if ctx.names.contains_const_recursively(var_name.as_ref()) {
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
    use super::*;
    use crate::common::{CanCastTo, QError, ToLocatableError};

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
