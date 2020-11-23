use crate::common::{AtLocation, CaseInsensitiveString, Locatable, QErrorNode};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::{
    BareName, DimNameNode, Expression, ExpressionNode, ExpressionType, FunctionMap, Name, NameNode,
    ParamNameNode, ParamNameNodes, QualifiedNameNode, SubMap, TypeQualifier, UserDefinedTypes,
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
    names_without_dot: HashSet<BareName>,
}

struct Names {
    compact_name_set: HashMap<BareName, HashSet<TypeQualifier>>,
    compact_names: HashMap<Name, ExpressionType>,
    extended_names: HashMap<BareName, ExpressionType>,
    constants: HashMap<BareName, Variant>,
    parent: Option<Box<Names>>,
}

impl Names {
    pub fn new(parent: Option<Box<Self>>) -> Self {
        Self {
            compact_name_set: HashMap::new(),
            compact_names: HashMap::new(),
            extended_names: HashMap::new(),
            constants: HashMap::new(),
            parent,
        }
    }

    pub fn contains_any<T: AsRef<BareName>>(&self, bare_name: T) -> bool {
        let x = bare_name.as_ref();
        self.compact_name_set.contains_key(x)
            || self.extended_names.contains_key(x)
            || self.constants.contains_key(x)
    }

    pub fn can_accept_compact<T: AsRef<BareName>>(
        &self,
        bare_name: T,
        qualifier: TypeQualifier,
    ) -> bool {
        let x = bare_name.as_ref();
        if self.extended_names.contains_key(x) || self.constants.contains_key(x) {
            false
        } else {
            match self.compact_name_set.get(x) {
                Some(qualifiers) => !qualifiers.contains(&qualifier),
                _ => true,
            }
        }
    }

    pub fn contains_compact(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&ExpressionType> {
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

    pub fn contains_extended(&self, bare_name: &BareName) -> Option<&ExpressionType> {
        self.extended_names.get(bare_name)
    }

    pub fn contains_const(&self, bare_name: &BareName) -> bool {
        self.constants.contains_key(bare_name)
    }

    pub fn get_const_value_no_recursion(&self, bare_name: &BareName) -> Option<&Variant> {
        self.constants.get(bare_name)
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
        expr_type: ExpressionType,
    ) {
        self.compact_names
            .insert(Name::new(bare_name.clone(), Some(q)), expr_type);
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

    pub fn insert_extended(&mut self, bare_name: BareName, expr_type: ExpressionType) {
        self.extended_names.insert(bare_name, expr_type);
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
            names_without_dot: HashSet::new(),
        }
    }

    pub fn push_sub_context(
        &mut self,
        params: ParamNameNodes,
    ) -> Result<ParamNameNodes, QErrorNode> {
        let temp_dummy = Names::new(None);
        let old_names = std::mem::replace(&mut self.names, temp_dummy);
        self.names = Names::new(Some(Box::new(old_names)));
        self.convert_param_name_nodes(params)
    }

    pub fn push_function_context(
        &mut self,
        name: Name,
        params: ParamNameNodes,
    ) -> Result<(Name, ParamNameNodes), QErrorNode> {
        let temp_dummy = Names::new(None);
        let old_names = std::mem::replace(&mut self.names, temp_dummy);
        self.names = Names::new(Some(Box::new(old_names)));
        Ok((name, self.convert_param_name_nodes(params)?))
    }

    pub fn pop_context(&mut self) {
        let temp_dummy = Names::new(None);
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
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        expr_rules::on_expression(self, expr_node)
    }

    pub fn on_assignment(
        &mut self,
        left_side: ExpressionNode,
        right_side: ExpressionNode,
    ) -> Result<(ExpressionNode, ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        assignment_pre_conversion_validation_rules::validate(self, &left_side)?;
        let (converted_right_side, right_side_implicit_vars) = self.on_expression(right_side)?;
        let (converted_left_side, left_side_implicit_vars) = self.on_expression(left_side)?;
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
    ) -> Result<(NameNode, ExpressionNode, Variant), QErrorNode> {
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
        let dim_name_node: DimNameNode = DimNameNode::from(param_name_node);
        let (converted_dim_name_node, implicits) = dim_rules::on_dim(self, dim_name_node, true)?;
        if implicits.is_empty() {
            Ok(ParamNameNode::from(converted_dim_name_node))
        } else {
            panic!("Should not have introduced implicit variables via parameter")
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
    use super::*;
    use crate::built_ins::BuiltInFunction;
    use crate::common::{QError, ToLocatableError};
    use crate::parser::{ExpressionNodes, HasExpressionType};
    use std::convert::TryFrom;

    type I = ExpressionNode;
    type O = (ExpressionNode, Vec<QualifiedNameNode>);
    type ExprResult = RuleResult<I, O>;
    type Result = std::result::Result<ExprResult, QErrorNode>;

    // TODO assignment to function unqualified or qualified by the function type

    pub fn on_expression(
        ctx: &mut Context,
        expr_node: ExpressionNode,
    ) -> std::result::Result<O, QErrorNode> {
        let conversion_rules = FnRule::new(literals)
            .chain_fn(name_clashes_with_sub)
            .chain_fn(existing_extended_var)
            .chain_fn(existing_const)
            .chain_fn(existing_extended_array_with_parenthesis)
            .chain_fn(existing_compact_array_with_parenthesis)
            .chain_fn(existing_compact_name)
            .chain_fn(unary_expr)
            .chain_fn(binary_expr)
            .chain_fn(implicit_var)
            .chain_fn(property_of_existing_var)
            .chain_fn(fold_property_into_implicit_var)
            .chain_fn(function_call_must_have_args)
            .chain_fn(function_call);
        conversion_rules.demand(ctx, expr_node)
    }

    fn name_clashes_with_sub(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, _),
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
            let bare_name = name.as_ref();
            if let Some(expr_type) = ctx.names.contains_extended(bare_name) {
                let converted_name = expr_type.qualify_name(name).with_err_at(pos)?;
                Ok(RuleResult::Success((
                    Expression::Variable(converted_name, expr_type.clone()).at(pos),
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

    fn existing_const(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, expr_type),
            pos,
        } = input
        {
            if let Some(v) = ctx.names.get_const_value_no_recursion(name.as_ref()) {
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

    fn existing_compact_name(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::Variable(name, expr_type),
            pos,
        } = input
        {
            let bare_name: &BareName = name.as_ref();
            let q: TypeQualifier = ctx.resolve_name_to_qualifier(&name);
            if let Some(expr_type) = ctx.names.contains_compact(bare_name, q) {
                let converted_name = name.qualify(q);
                let expr = Expression::Variable(converted_name, expr_type.clone());
                Ok(RuleResult::Success((expr.at(pos), vec![])))
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

    fn binary_expr(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::BinaryExpression(op, left, right, _),
            pos,
        } = input
        {
            let (converted_left, left_implicit_vars) = ctx.on_expression(*left)?;
            let (converted_right, right_implicit_vars) = ctx.on_expression(*right)?;
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
            let (converted_child, implicit_vars) = ctx.on_expression(*child)?;
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
            ctx.names
                .insert_compact(resolved_name.as_ref().clone(), qualifier, expr_type.clone());
            let resoled_expr = Expression::Variable(resolved_name, expr_type).at(pos);
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
                if let Some(ExpressionType::UserDefined(_)) =
                    ctx.names.contains_extended(left_most_name.as_ref())
                {
                    let (
                        Locatable {
                            element: converted_left,
                            ..
                        },
                        implicit_left,
                    ) = ctx.on_expression((*left).at(pos))?;
                    let converted_left_expr_type = converted_left.expression_type();
                    let converted_left_element_type = converted_left_expr_type.to_element_type();
                    if let ExpressionType::UserDefined(user_defined_type_name) =
                        converted_left_element_type
                    {
                        if let Some(user_defined_type) =
                            ctx.user_defined_types.get(user_defined_type_name)
                        {
                            if let Some(element_type) =
                                user_defined_type.find_element(right.as_ref())
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

    fn existing_extended_array_with_parenthesis(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::FunctionCall(name, args),
            pos,
        } = input
        {
            let bare_name = name.as_ref();
            if let Some(ExpressionType::Array(boxed_element_type, _)) =
                ctx.names.contains_extended(bare_name)
            {
                // clone element type early in order to be able to use ctx as mutable later
                let element_type = boxed_element_type.as_ref().clone();
                // convert args
                let mut implicit_vars: Vec<QualifiedNameNode> = vec![];
                let mut converted_args: ExpressionNodes = vec![];
                for arg in args {
                    let (converted_arg, implicits) = ctx.on_expression(arg)?;
                    converted_args.push(converted_arg);
                    implicit_vars = union(implicit_vars, implicits);
                }
                // convert name
                let converted_name = element_type.qualify_name(name).with_err_at(pos)?;
                // create result
                let result_expr = if converted_args.is_empty() {
                    // just the array itself. It is with parenthesis, as we are in a function call
                    Expression::Variable(
                        converted_name,
                        ExpressionType::Array(Box::new(element_type), true),
                    )
                } else {
                    Expression::ArrayElement(converted_name, converted_args, element_type)
                };
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

    // TODO : merge this function with extended variation
    fn existing_compact_array_with_parenthesis(ctx: &mut Context, input: I) -> Result {
        if let Locatable {
            element: Expression::FunctionCall(name, args),
            pos,
        } = input
        {
            let bare_name: &BareName = name.as_ref();
            let qualifier: TypeQualifier = ctx.resolve_name_to_qualifier(&name);
            if let Some(ExpressionType::Array(boxed_element_type, _)) =
                ctx.names.contains_compact(bare_name, qualifier)
            {
                // clone element type early in order to be able to use ctx as mutable later
                let element_type = boxed_element_type.as_ref().clone();
                // convert args
                let mut implicit_vars: Vec<QualifiedNameNode> = vec![];
                let mut converted_args: ExpressionNodes = vec![];
                for arg in args {
                    let (converted_arg, implicits) = ctx.on_expression(arg)?;
                    converted_args.push(converted_arg);
                    implicit_vars = union(implicit_vars, implicits);
                }
                // convert name
                let converted_name = name.qualify(qualifier);
                // create result
                let result_expr = if converted_args.is_empty() {
                    // just the array itself. It is with parenthesis, as we are in a function call
                    Expression::Variable(
                        converted_name,
                        ExpressionType::Array(Box::new(element_type), true),
                    )
                } else {
                    Expression::ArrayElement(converted_name, converted_args, element_type)
                };
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
            let mut implicit_vars: Vec<QualifiedNameNode> = vec![];
            let mut converted_args: ExpressionNodes = vec![];
            for arg in args {
                let (converted_arg, implicits) = ctx.on_expression(arg)?;
                converted_args.push(converted_arg);
                implicit_vars = union(implicit_vars, implicits);
            }
            // is it built-in function?
            let converted_expr =
                match Option::<BuiltInFunction>::try_from(&name).with_err_at(pos)? {
                    Some(built_in_function) => {
                        Expression::BuiltInFunctionCall(built_in_function, converted_args)
                    }
                    _ => {
                        let converted_name: Name = match ctx.functions.get(name.as_ref()) {
                            Some(Locatable {
                                element: (q, _), ..
                            }) => name.qualify(*q),
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
}

pub mod dim_rules {
    use super::*;
    use crate::common::{QError, ToLocatableError};
    use crate::parser::{
        ArrayDimension, ArrayDimensions, BuiltInStyle, DimName, DimType, DimTypeTrait,
        HasExpressionType,
    };
    use crate::variant::MAX_INTEGER;
    use std::convert::TryFrom;

    type I = DimNameNode;
    type O = (DimNameNode, Vec<QualifiedNameNode>);
    type DimResult = RuleResult<I, O>;
    type Result = std::result::Result<DimResult, QErrorNode>;

    pub fn on_dim(
        ctx: &mut Context,
        dim_name_node: DimNameNode,
        is_param: bool,
    ) -> std::result::Result<(DimNameNode, Vec<QualifiedNameNode>), QErrorNode> {
        let rule = FnRule::new(cannot_clash_with_subs)
            .chain_fn(if is_param {
                cannot_clash_with_functions_param
            } else {
                cannot_clash_with_functions_dim
            })
            .chain_fn(cannot_clash_with_existing_names)
            .chain_fn(user_defined_type_must_exist)
            .chain_fn(new_var);
        rule.demand(ctx, dim_name_node)
    }

    fn user_defined_type_must_exist(ctx: &mut Context, input: I) -> Result {
        if let Some(user_defined_type_name_node) = input.is_user_defined() {
            if ctx
                .user_defined_types
                .contains_key(user_defined_type_name_node.as_ref())
            {
                Ok(RuleResult::Skip(input))
            } else {
                Err(QError::TypeNotDefined).with_err_at(user_defined_type_name_node)
            }
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn cannot_clash_with_functions_dim(ctx: &mut Context, input: I) -> Result {
        if ctx.functions.contains_key(input.as_ref()) {
            Err(QError::DuplicateDefinition).with_err_at(&input)
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn cannot_clash_with_functions_param(ctx: &mut Context, input: I) -> Result {
        match ctx.functions.get(input.as_ref()) {
            Some(Locatable {
                element: (func_qualifier, _),
                ..
            }) => {
                if input.is_extended() {
                    Err(QError::DuplicateDefinition).with_err_at(&input)
                } else {
                    let q = ctx.resolve_name_ref_to_qualifier(&input);
                    if q == *func_qualifier {
                        // for some reason you can have a FUNCTION Add(Add)
                        Ok(RuleResult::Skip(input))
                    } else {
                        Err(QError::DuplicateDefinition).with_err_at(&input)
                    }
                }
            }
            _ => Ok(RuleResult::Skip(input)),
        }
    }

    fn cannot_clash_with_subs(ctx: &mut Context, input: I) -> Result {
        if ctx.subs.contains_key(input.as_ref()) {
            Err(QError::DuplicateDefinition).with_err_at(&input)
        } else {
            Ok(RuleResult::Skip(input))
        }
    }

    fn cannot_clash_with_existing_names(ctx: &mut Context, input: I) -> Result {
        if input.is_extended() {
            if ctx.names.contains_any(&input) {
                Err(QError::DuplicateDefinition).with_err_at(&input)
            } else {
                Ok(RuleResult::Skip(input))
            }
        } else {
            let qualifier = ctx.resolve_name_ref_to_qualifier(&input);
            if ctx.names.can_accept_compact(&input, qualifier) {
                Ok(RuleResult::Skip(input))
            } else {
                Err(QError::DuplicateDefinition).with_err_at(&input)
            }
        }
    }

    fn new_var(ctx: &mut Context, input: I) -> Result {
        let (converted_input, implicit_vars) = new_var_not_adding_to_context(ctx, input)?;
        // add to context
        let bare_name: &BareName = converted_input.as_ref();
        let expr_type = converted_input.expression_type();
        if converted_input.is_extended() {
            ctx.names.insert_extended(bare_name.clone(), expr_type);
        } else {
            let q = TypeQualifier::try_from(&converted_input)?;
            ctx.names.insert_compact(bare_name.clone(), q, expr_type);
        }
        Ok(RuleResult::Success((converted_input, implicit_vars)))
    }

    fn new_var_not_adding_to_context(
        ctx: &mut Context,
        input: I,
    ) -> std::result::Result<O, QErrorNode> {
        let Locatable {
            element: dim_name,
            pos,
        } = input;
        let DimName {
            bare_name,
            dim_type,
        } = dim_name;
        match dim_type {
            DimType::Bare => {
                let qualifier = ctx.resolve(&bare_name);
                let dim_type = DimType::BuiltIn(qualifier, BuiltInStyle::Compact);
                let converted_dim_name = DimName::new(bare_name, dim_type);
                Ok((converted_dim_name.at(pos), vec![]))
            }
            DimType::BuiltIn(q, built_in_style) => {
                let result = DimName::new(bare_name, DimType::BuiltIn(q, built_in_style)).at(pos);
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
                let result =
                    DimName::new(bare_name, DimType::UserDefined(user_defined_type_name_node))
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
                        Some(lbound) => ctx.on_expression(lbound).map(|(x, y)| (Some(x), y))?,
                        _ => (None, vec![]),
                    };
                    let (converted_ubound, implicit_vars_ubound) = ctx.on_expression(ubound)?;
                    converted_dimensions.push(ArrayDimension {
                        lbound: converted_lbound,
                        ubound: converted_ubound,
                    });
                    implicit_vars = union(implicit_vars, implicit_vars_lbound);
                    implicit_vars = union(implicit_vars, implicit_vars_ubound);
                }
                // dim_type
                let element_dim_type = *boxed_element_type;
                let element_dim_name = DimName::new(bare_name.clone(), element_dim_type).at(pos);
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
                    DimName::new(bare_name, array_dim_type).at(pos),
                    implicit_vars,
                ))
            }
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
        if ctx.names.contains_any(&const_name)
            || ctx.subs.contains_key(const_name.as_ref())
            || ctx.functions.contains_key(const_name.as_ref())
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
                pos: const_name_pos,
            },
            right,
        ) = input;
        let v = ctx.names.resolve_const_value_node(&right)?;
        let q = TypeQualifier::try_from(&v).with_err_at(&right)?;
        if const_name.is_bare_or_of_type(q) {
            ctx.names
                .insert_const(const_name.as_ref().clone(), v.clone());
            let converted_name = const_name.qualify(q).at(const_name_pos);
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
    use crate::common::{QError, ToLocatableError};

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
