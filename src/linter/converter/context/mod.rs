mod dim_rules;
mod expr_rules;
mod names;

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;

use crate::common::{Locatable, QErrorNode};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::{DimContext, NameContext};
use crate::parser::*;
use names::Names;

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExprContext {
    Default,
    Assignment,
    Parameter,

    /// Used in resolving left-side of property expressions
    ResolvingPropertyOwner,
}

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
        dim_list: DimList,
        dim_context: DimContext,
    ) -> Result<(DimList, Vec<QualifiedNameNode>), QErrorNode> {
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
    use super::*;
    use crate::common::{QError, ToLocatableError};
    use std::convert::TryFrom;

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
    use super::*;
    use crate::common::{QError, ToLocatableError};

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
    use super::*;
    use crate::common::{CanCastTo, QError, ToLocatableError};

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

fn union(
    mut left: Vec<QualifiedNameNode>,
    mut right: Vec<QualifiedNameNode>,
) -> Vec<QualifiedNameNode> {
    left.append(&mut right);
    left
}
