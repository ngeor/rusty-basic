//! Converter is the main logic of the linter, where most validation takes place,
//! as well as resolving variable types.
mod assignment;
mod const_rules;
mod conversion_traits;
mod dim_rules;
mod do_loop;
mod expr_rules;
mod for_loop;
mod function_implementation;
mod if_blocks;
mod names;
mod print_node;
mod select_case;
mod statement;
mod sub_call;
mod sub_implementation;
mod top_level_token;

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
use conversion_traits::SameTypeConverter;
use names::Names;

pub fn convert(
    program: ProgramNode,
    f_c: &FunctionMap,
    s_c: &SubMap,
    user_defined_types: &UserDefinedTypes,
) -> Result<(ProgramNode, HashSet<BareName>), QErrorNode> {
    let mut converter = ConverterImpl::new(user_defined_types, f_c, s_c);
    let result = converter.convert_same_type(program)?;
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

/// Resolves all symbols of the program, converting it into an explicitly typed
/// equivalent program.
struct ConverterImpl<'a> {
    pub resolver: Rc<RefCell<TypeResolverImpl>>,
    // TODO pass a trait that exposes only the functionality really needed here
    pub functions: &'a FunctionMap,
    // TODO pass a trait that exposes only the functionality really needed here
    pub subs: &'a SubMap,
    // TODO pass a trait that exposes only the functionality really needed here
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
            let (converted_expr_node, mut implicits) =
                self.on_expression(expr_node, expr_context)?;
            converted_expr_nodes.push(converted_expr_node);
            implicit_vars.append(&mut implicits);
        }
        Ok((converted_expr_nodes, implicit_vars))
    }

    pub fn on_assignment(
        &mut self,
        left_side: ExpressionNode,
        right_side: ExpressionNode,
    ) -> Result<(ExpressionNode, ExpressionNode, Implicits), QErrorNode> {
        assignment::on_assignment(self, left_side, right_side)
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
