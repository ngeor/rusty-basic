use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use super::assignment;
use super::const_rules;
use super::dim_rules;
use super::expr_rules;
use super::names::Names;
use crate::common::*;
use crate::linter::converter::conversion_traits::SameTypeConverter;
use crate::linter::pre_linter::PreLinterResult;
use crate::linter::type_resolver::{IntoQualified, TypeResolver};
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::{
    DimContext, FunctionMap, HasFunctions, HasSubs, HasUserDefinedTypes, NameContext, SubMap,
};
use crate::parser::*;

pub fn convert(
    program: ProgramNode,
    pre_linter_result: Rc<PreLinterResult>,
) -> Result<(ProgramNode, HashSet<BareName>), QErrorNode> {
    let mut converter = ConverterImpl::new(pre_linter_result);
    let result = converter.convert_program(program)?;
    // consume
    let names_without_dot = converter.consume();
    Ok((result, names_without_dot))
}

/// Alias for the implicit variables collected during evaluating something.
/// e.g. `INPUT N` is a statement implicitly defining variable `N`.
pub type Implicits = Vec<QualifiedNameNode>;

/// Resolves all symbols of the program, converting it into an explicitly typed
/// equivalent program.
pub struct ConverterImpl {
    pub resolver: Rc<RefCell<TypeResolverImpl>>,
    pub context: Context,
}

impl ConverterImpl {
    pub fn new(pre_linter_result: Rc<PreLinterResult>) -> Self {
        let resolver = Rc::new(RefCell::new(TypeResolverImpl::new()));
        Self {
            resolver: Rc::clone(&resolver),
            context: Context::new(pre_linter_result, Rc::clone(&resolver)),
        }
    }

    pub fn consume(self) -> HashSet<BareName> {
        self.context.names_without_dot()
    }

    pub fn convert_block_removing_constants(
        &mut self,
        statements: StatementNodes,
    ) -> Result<StatementNodes, QErrorNode> {
        let mut result: StatementNodes = vec![];
        for statement in statements {
            let converted_statement_node = self.convert(statement)?;
            if let Statement::Const(_, _) = converted_statement_node.as_ref() {
                // filter out CONST statements, they've been registered into context as values
            } else {
                result.push(converted_statement_node);
            }
        }
        Ok(result)
    }

    // A statement can be expanded into multiple statements to convert implicitly
    // declared variables into explicit.
    // Example:
    //      A = B + C
    // becomes
    //      DIM B
    //      DIM C
    //      DIM A
    //      A = B + C
    pub fn convert_block_hoisting_implicits(
        &mut self,
        statements: StatementNodes,
    ) -> Result<StatementNodes, QErrorNode> {
        let mut result = self.convert_block_removing_constants(statements)?;

        let implicits = self.context.pop_context();
        let mut implicit_dim: StatementNodes = implicits
            .into_iter()
            .map(
                |Locatable {
                     element: q_name,
                     pos,
                 }| Statement::Dim(DimName::from(q_name).into_list(pos)).at(pos),
            )
            .collect();

        implicit_dim.append(&mut result);
        Ok(implicit_dim)
    }

    pub fn convert_program(&mut self, program: ProgramNode) -> Result<ProgramNode, QErrorNode> {
        let mut result: ProgramNode = vec![];
        for Locatable { element, pos } in program {
            match element {
                TopLevelToken::DefType(def_type) => {
                    self.resolver.borrow_mut().set(&def_type);
                }
                TopLevelToken::FunctionDeclaration(_name, _params) => {}
                TopLevelToken::FunctionImplementation(f) => {
                    let converted = self.convert_function_implementation(f)?;
                    result.push(converted.at(pos));
                }
                TopLevelToken::Statement(s) => {
                    let converted = self.convert_block_removing_constants(vec![s.at(pos)])?;
                    result.extend(
                        converted
                            .into_iter()
                            .map(|statement_node| statement_node.map(TopLevelToken::Statement)),
                    );
                }
                TopLevelToken::SubDeclaration(_name, _params) => {}
                TopLevelToken::SubImplementation(s) => {
                    let converted = self.convert_sub_implementation(s)?;
                    result.push(converted.at(pos));
                }
                TopLevelToken::UserDefinedType(_) => {
                    // already handled at first pass
                }
            }
        }

        // insert implicits at the top
        let mut implicits = Implicits::new();
        implicits.append(self.context.names.get_implicits());
        let mut implicit_statements: ProgramNode = implicits
            .into_iter()
            .map(|Locatable { element, pos }| {
                TopLevelToken::Statement(Statement::Dim(DimName::from(element).into_list(pos)))
                    .at(pos)
            })
            .collect();
        implicit_statements.append(&mut result);
        Ok(implicit_statements)
    }
}

impl TypeResolver for ConverterImpl {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.borrow().char_to_qualifier(ch)
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

// TODO visibility creep due to mod reorg

pub struct Context {
    pre_linter_result: Rc<PreLinterResult>,
    resolver: Rc<RefCell<TypeResolverImpl>>,
    pub names: Names,
    names_without_dot: HashSet<BareName>,
}

impl TypeResolver for Context {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.borrow().char_to_qualifier(ch)
    }
}

impl HasFunctions for Context {
    fn functions(&self) -> &FunctionMap {
        self.pre_linter_result.functions()
    }
}

impl HasSubs for Context {
    fn subs(&self) -> &SubMap {
        self.pre_linter_result.subs()
    }
}

impl HasUserDefinedTypes for Context {
    fn user_defined_types(&self) -> &UserDefinedTypes {
        self.pre_linter_result.user_defined_types()
    }
}

impl Context {
    pub fn new(
        pre_linter_result: Rc<PreLinterResult>,
        resolver: Rc<RefCell<TypeResolverImpl>>,
    ) -> Self {
        Self {
            pre_linter_result,
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
        let converted_function_name = name.to_qualified(self);
        Ok((converted_function_name, dim_rules::on_params(self, params)?))
    }

    pub fn pop_context(&mut self) -> Implicits {
        // temp object for mem swap
        let temp_dummy = Names::new_root();
        // take current "self.names" and store into "current"
        let mut current = std::mem::replace(&mut self.names, temp_dummy);
        // collect extended names of sub-program, as they can't be combined with dots anywhere in the program
        current.drain_extended_names_into(&mut self.names_without_dot);
        // collect implicits
        let mut implicits = Implicits::new();
        implicits.append(current.get_implicits());
        // set parent as current
        self.names = current.pop_parent().expect("Stack underflow");
        implicits
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
    ) -> Result<ExpressionNode, QErrorNode> {
        expr_rules::on_expression(self, expr_node, expr_context)
    }

    pub fn on_opt_expression(
        &mut self,
        opt_expr_node: Option<ExpressionNode>,
        expr_context: ExprContext,
    ) -> Result<Option<ExpressionNode>, QErrorNode> {
        match opt_expr_node {
            Some(expr_node) => self.on_expression(expr_node, expr_context).map(Some),
            _ => Ok(None),
        }
    }

    pub fn on_expressions(
        &mut self,
        expr_nodes: ExpressionNodes,
        expr_context: ExprContext,
    ) -> Result<ExpressionNodes, QErrorNode> {
        let mut converted_expr_nodes: ExpressionNodes = vec![];
        for expr_node in expr_nodes {
            let converted_expr_node = self.on_expression(expr_node, expr_context)?;
            converted_expr_nodes.push(converted_expr_node);
        }
        Ok(converted_expr_nodes)
    }

    pub fn on_assignment(
        &mut self,
        left_side: ExpressionNode,
        right_side: ExpressionNode,
    ) -> Result<(ExpressionNode, ExpressionNode), QErrorNode> {
        assignment::on_assignment(self, left_side, right_side)
    }

    pub fn on_dim(
        &mut self,
        dim_list: DimList,
        dim_context: DimContext,
    ) -> Result<DimList, QErrorNode> {
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
    pub fn function_qualifier(&self, bare_name: &BareName) -> Option<TypeQualifier> {
        self.functions().get(bare_name).map(
            |Locatable {
                 element: (q, _), ..
             }| *q,
        )
    }
}
