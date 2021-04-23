use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::common::*;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::{DimContext, NameContext};
use crate::parser::{
    BareName, DimList, ExpressionNode, ExpressionNodes, FunctionMap, Name, NameNode,
    ParamNameNodes, ProgramNode, QualifiedNameNode, SubMap, TypeQualifier, UserDefinedTypes,
};

mod assignment;
mod dim_rules;
mod do_loop;
mod expr_rules;
mod for_loop;
mod function_implementation;
mod if_blocks;
mod names;
mod print_node;
mod program;
mod select_case;
mod statement;
mod sub_call;
mod sub_implementation;
mod top_level_token;

pub fn convert(
    program: ProgramNode,
    f_c: &FunctionMap,
    s_c: &SubMap,
    user_defined_types: &UserDefinedTypes,
) -> Result<(ProgramNode, HashSet<BareName>), QErrorNode> {
    let mut converter = ConverterImpl::new(user_defined_types, f_c, s_c);
    let result = converter.convert(program)?;
    // consume
    let names_without_dot = converter.consume();
    Ok((result, names_without_dot))
}

/// Alias for the implicit variables collected during evaluating something.
/// e.g. `INPUT N` is a statement implicitly defining variable `N`.
type Implicits = Vec<QualifiedNameNode>;

/// Alias for the result of returning something together with any implicit
/// variables collected during its conversion.
type R<T> = Result<(T, Implicits), QErrorNode>;

//
// Converter trait
//

trait Converter<A, B> {
    fn convert(&mut self, a: A) -> Result<B, QErrorNode>;
}

// blanket for Vec
impl<T, A, B> Converter<Vec<A>, Vec<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Vec<A>) -> Result<Vec<B>, QErrorNode> {
        a.into_iter().map(|x| self.convert(x)).collect()
    }
}

// blanket for Option
impl<T, A, B> Converter<Option<A>, Option<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Option<A>) -> Result<Option<B>, QErrorNode> {
        match a {
            Some(x) => self.convert(x).map(|r| Some(r)),
            None => Ok(None),
        }
    }
}

// blanket for Locatable
impl<T, A, B> Converter<Locatable<A>, Locatable<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Locatable<A>) -> Result<Locatable<B>, QErrorNode> {
        let Locatable { element, pos } = a;
        self.convert(element).map(|x| x.at(pos)).patch_err_pos(pos)
    }
}

//
// ConverterWithImplicitVariables
//

trait ConverterWithImplicitVariables<A, B> {
    fn convert_and_collect_implicit_variables(&mut self, a: A) -> R<B>;
}

// blanket for Option

impl<T, A, B> ConverterWithImplicitVariables<Option<A>, Option<B>> for T
where
    T: ConverterWithImplicitVariables<A, B>,
{
    fn convert_and_collect_implicit_variables(&mut self, a: Option<A>) -> R<Option<B>> {
        match a {
            Some(a) => self
                .convert_and_collect_implicit_variables(a)
                .map(|(a, implicit_variables)| (Some(a), implicit_variables)),
            None => Ok((None, vec![])),
        }
    }
}

// blanket for Box

impl<T, A, B> ConverterWithImplicitVariables<Box<A>, Box<B>> for T
where
    T: ConverterWithImplicitVariables<A, B>,
{
    fn convert_and_collect_implicit_variables(&mut self, a: Box<A>) -> R<Box<B>> {
        let unboxed: A = *a;
        let (converted, implicit_variables) =
            self.convert_and_collect_implicit_variables(unboxed)?;
        Ok((Box::new(converted), implicit_variables))
    }
}

// blanket for Vec

impl<T, A, B> ConverterWithImplicitVariables<Vec<A>, Vec<B>> for T
where
    T: ConverterWithImplicitVariables<A, B>,
{
    fn convert_and_collect_implicit_variables(&mut self, a: Vec<A>) -> R<Vec<B>> {
        let mut result: Vec<B> = vec![];
        let mut total_implicit: Implicits = vec![];
        for i in a {
            let (b, mut implicit) = self.convert_and_collect_implicit_variables(i)?;
            result.push(b);
            total_implicit.append(&mut implicit);
        }
        Ok((result, total_implicit))
    }
}

//
// Converter
//

struct ConverterImpl<'a> {
    pub resolver: Rc<RefCell<TypeResolverImpl>>,
    pub functions: &'a FunctionMap,
    pub subs: &'a SubMap,
    pub user_defined_types: &'a UserDefinedTypes,
    pub context: Context<'a>,
}

impl<'a> ConverterImpl<'a> {
    pub fn new(
        user_defined_types: &'a UserDefinedTypes,
        functions: &'a FunctionMap,
        subs: &'a SubMap,
    ) -> Self {
        let resolver = Rc::new(RefCell::new(TypeResolverImpl::new()));
        Self {
            user_defined_types,
            resolver: Rc::clone(&resolver),
            functions,
            subs,
            context: Context::new(functions, subs, user_defined_types, Rc::clone(&resolver)),
        }
    }

    pub fn consume(self) -> HashSet<BareName> {
        self.context.names_without_dot()
    }

    pub fn merge_implicit_vars(lists: Vec<Implicits>) -> Implicits {
        let mut result: Implicits = vec![];
        for mut list in lists {
            result.append(&mut list);
        }
        result
    }
}

impl<'a> TypeResolver for ConverterImpl<'a> {
    fn resolve_char(&self, ch: char) -> TypeQualifier {
        self.resolver.borrow().resolve_char(ch)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExprContext {
    Default,
    Assignment,
    Parameter,

    /// Used in resolving left-side of property expressions
    ResolvingPropertyOwner,
}

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

use crate::linter::const_value_resolver::ConstValueResolver;
use names::Names;

pub struct Context<'a> {
    functions: &'a FunctionMap,
    subs: &'a SubMap,
    user_defined_types: &'a UserDefinedTypes,
    resolver: Rc<RefCell<TypeResolverImpl>>,
    names: Names,
    names_without_dot: HashSet<BareName>,
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
        dim_rules::on_params(self, params)
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
        Ok((converted_function_name, dim_rules::on_params(self, params)?))
    }

    pub fn pop_context(&mut self) {
        // temp object for mem swap
        let temp_dummy = Names::new_root();
        // take current "self.names" and store into "current"
        let mut current = std::mem::replace(&mut self.names, temp_dummy);
        // collect extended names of sub-program, as they can't be combined with dots anywhere in the program
        current.drain_extended_names_into(&mut self.names_without_dot);
        // set parent as current
        self.names = current.pop_parent().expect("Stack underflow");
    }

    pub fn is_in_subprogram(&self) -> bool {
        self.names.is_in_subprogram()
    }

    pub fn get_name_context(&self) -> NameContext {
        self.names.get_name_context()
    }

    pub fn names_without_dot(mut self) -> HashSet<BareName> {
        self.names
            .drain_extended_names_into(&mut self.names_without_dot);
        self.names_without_dot
    }

    pub fn on_expression(
        &mut self,
        expr_node: ExpressionNode,
        expr_context: ExprContext,
    ) -> R<ExpressionNode> {
        expr_rules::on_expression(self, expr_node, expr_context)
    }

    pub fn on_opt_expression(
        &mut self,
        opt_expr_node: Option<ExpressionNode>,
        expr_context: ExprContext,
    ) -> R<Option<ExpressionNode>> {
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
    ) -> R<ExpressionNodes> {
        let mut implicit_vars: Implicits = vec![];
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
    ) -> Result<(ExpressionNode, ExpressionNode, Implicits), QErrorNode> {
        assignment_pre_conversion_validation_rules::validate(self, &left_side)?;
        let (converted_right_side, right_side_implicit_vars) =
            self.on_expression(right_side, ExprContext::Default)?;
        let (converted_left_side, left_side_implicit_vars) =
            expr_rules::on_expression(self, left_side, ExprContext::Assignment)?;
        assignment_post_conversion_validation_rules::validate(
            &converted_left_side,
            &converted_right_side,
        )?;
        Ok((
            converted_left_side,
            converted_right_side,
            union(left_side_implicit_vars, right_side_implicit_vars),
        ))
    }

    pub fn on_dim(&mut self, dim_list: DimList, dim_context: DimContext) -> R<DimList> {
        dim_rules::on_dim(self, dim_list, dim_context)
    }

    pub fn on_const(
        &mut self,
        left_side: NameNode,
        right_side: ExpressionNode,
    ) -> Result<(), QErrorNode> {
        const_rules::on_const(self, left_side, right_side)
    }

    /// Gets the function qualifier of the function identified by the given bare name.
    /// If no such function exists, returns `None`.
    fn function_qualifier(&self, bare_name: &BareName) -> Option<TypeQualifier> {
        self.functions.get(bare_name).map(
            |Locatable {
                 element: (q, _), ..
             }| *q,
        )
    }
}

mod const_rules {
    use std::convert::TryFrom;

    use crate::common::{QError, ToLocatableError};

    use super::*;

    pub fn on_const(
        ctx: &mut Context,
        left_side: NameNode,
        right_side: ExpressionNode,
    ) -> Result<(), QErrorNode> {
        const_cannot_clash_with_existing_names(ctx, &left_side)?;
        new_const(ctx, left_side, right_side)
    }

    fn const_cannot_clash_with_existing_names(
        ctx: &mut Context,
        left_side: &NameNode,
    ) -> Result<(), QErrorNode> {
        let Locatable {
            element: const_name,
            pos: const_name_pos,
        } = left_side;
        if ctx
            .names
            .contains_any_locally_or_contains_extended_recursively(const_name.bare_name())
            || ctx.subs.contains_key(const_name.bare_name())
            || ctx.functions.contains_key(const_name.bare_name())
        {
            Err(QError::DuplicateDefinition).with_err_at(*const_name_pos)
        } else {
            Ok(())
        }
    }

    fn new_const(
        ctx: &mut Context,
        left_side: NameNode,
        right_side: ExpressionNode,
    ) -> Result<(), QErrorNode> {
        let Locatable {
            element: const_name,
            ..
        } = left_side;
        let value_before_casting = ctx.names.resolve_const_value_node(&right_side)?;
        let value_qualifier =
            TypeQualifier::try_from(&value_before_casting).with_err_at(&right_side)?;
        let final_value = if const_name.is_bare_or_of_type(value_qualifier) {
            value_before_casting
        } else {
            value_before_casting
                .cast(const_name.qualifier().unwrap())
                .with_err_at(&right_side)?
        };
        ctx.names
            .insert_const(const_name.bare_name().clone(), final_value.clone());
        Ok(())
    }
}

mod assignment_pre_conversion_validation_rules {
    use crate::common::{QError, ToLocatableError};
    use crate::parser::Expression;

    use super::*;

    pub fn validate(ctx: &mut Context, left_side: &ExpressionNode) -> Result<(), QErrorNode> {
        cannot_assign_to_const(ctx, left_side)
    }

    fn cannot_assign_to_const(ctx: &mut Context, input: &ExpressionNode) -> Result<(), QErrorNode> {
        if let Locatable {
            element: Expression::Variable(var_name, _),
            ..
        } = input
        {
            if ctx.names.contains_const_recursively(var_name.bare_name()) {
                Err(QError::DuplicateDefinition).with_err_at(input)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

mod assignment_post_conversion_validation_rules {
    use crate::common::{CanCastTo, QError, ToLocatableError};

    use super::*;

    pub fn validate(
        left_side: &ExpressionNode,
        right_side: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        if right_side.can_cast_to(left_side) {
            Ok(())
        } else {
            Err(QError::TypeMismatch).with_err_at(right_side)
        }
    }
}

fn union(mut left: Implicits, mut right: Implicits) -> Implicits {
    left.append(&mut right);
    left
}
