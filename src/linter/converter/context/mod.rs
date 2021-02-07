mod expr_rules;
mod names;

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;

use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::*;
use crate::variant::Variant;
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
        // temp object for mem swap
        let temp_dummy = Names::new_root();
        // take current "self.names" and store into "current"
        let mut current = std::mem::replace(&mut self.names, temp_dummy);
        // collect extended names of sub-program, as they can't be combined with dots anywhere in the program
        current.drain_extended_names_into(&mut self.names_without_dot);
        // set parent as current
        self.names = current.pop_parent().expect("Stack underflow");
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
            if ctx.names.is_in_subprogram() && dim_name_node.shared {
                Err(QError::syntax_error("SHARED not allowed in subprogram"))
                    .with_err_at(dim_name_node)
            } else {
                Ok(())
            }
        }
    }
}

pub mod const_rules {
    use super::*;
    use crate::common::{QError, ToLocatableError};
    use std::convert::TryFrom;

    pub fn on_const(
        ctx: &mut Context,
        left_side: NameNode,
        right_side: ExpressionNode,
    ) -> Result<(), QErrorNode> {
        // TODO merge with next rule
        if ctx
            .names
            .get_extended_var_recursively(left_side.bare_name())
            .is_some()
        {
            return Err(QError::DuplicateDefinition).with_err_at(&left_side);
        }

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
            .contains_local_var_or_local_const(const_name.bare_name())
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

pub mod assignment_pre_conversion_validation_rules {
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

pub mod assignment_post_conversion_validation_rules {
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