use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use super::assignment;
use super::const_rules;
use super::dim_rules;
use super::expr_rules;
use super::names::Names;
use crate::common::*;
use crate::linter::converter::conversion_traits::SameTypeConverterWithImplicits;
use crate::linter::pre_linter::{HasFunctions, HasSubs, HasUserDefinedTypes, PreLinterResult};
use crate::linter::type_resolver::{IntoQualified, TypeResolver};
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::{DimContext, FunctionMap, NameContext, SubMap};
use crate::parser::*;

pub fn convert(
    program: ProgramNode,
    pre_linter_result: &PreLinterResult,
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

/// Alias for the result of returning something together with any implicit
/// variables collected during its conversion.
pub type R<T> = Result<(T, Implicits), QErrorNode>;

/// Resolves all symbols of the program, converting it into an explicitly typed
/// equivalent program.
pub struct ConverterImpl<'a> {
    pub resolver: Rc<RefCell<TypeResolverImpl>>,
    pub context: Context<'a>,
}

impl<'a> ConverterImpl<'a> {
    pub fn new(pre_linter_result: &'a PreLinterResult) -> Self {
        let resolver = Rc::new(RefCell::new(TypeResolverImpl::new()));
        Self {
            resolver: Rc::clone(&resolver),
            context: Context::new(pre_linter_result, Rc::clone(&resolver)),
        }
    }

    pub fn consume(self) -> HashSet<BareName> {
        self.context.names_without_dot()
    }

    pub fn convert_block_keeping_implicits(
        &mut self,
        statements: StatementNodes,
    ) -> R<StatementNodes> {
        let mut result: StatementNodes = vec![];
        let mut implicits: Implicits = vec![];
        for statement in statements {
            let (converted_statement_node, mut part_implicits) =
                self.convert_same_type_with_implicits(statement)?;
            if let Statement::Const(_, _) = converted_statement_node.as_ref() {
                // filter out CONST statements, they've been registered into context as values
                debug_assert!(
                    part_implicits.is_empty(),
                    "Should not introduce implicits in a CONST"
                );
            } else {
                result.push(converted_statement_node);
                implicits.append(&mut part_implicits);
            }
        }
        Ok((result, implicits))
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
        let (mut result, implicits) = self.convert_block_keeping_implicits(statements)?;
        let mut index = 0;
        for implicit_var in implicits {
            let Locatable {
                element: q_name,
                pos,
            } = implicit_var;
            result.insert(
                index,
                Statement::Dim(DimName::from(q_name).into_list(pos)).at(pos),
            );
            index += 1;
        }
        Ok(result)
    }

    pub fn convert_program(&mut self, program: ProgramNode) -> Result<ProgramNode, QErrorNode> {
        let mut result: ProgramNode = vec![];
        let mut index: usize = 0;
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
                    let (converted, implicits) =
                        self.convert_block_keeping_implicits(vec![s.at(pos)])?;
                    // insert implicits at the top
                    for Locatable { element, pos } in implicits {
                        let implicit_statement =
                            Statement::Dim(DimName::from(element).into_list(pos));
                        result.insert(index, TopLevelToken::Statement(implicit_statement).at(pos));
                        index += 1;
                    }
                    // insert statements
                    for Locatable { element, pos } in converted {
                        result.push(TopLevelToken::Statement(element).at(pos));
                    }
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
        Ok(result)
    }
}

impl<'a> TypeResolver for ConverterImpl<'a> {
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

pub struct Context<'a> {
    pre_linter_result: &'a PreLinterResult,
    resolver: Rc<RefCell<TypeResolverImpl>>,
    pub names: Names,
    names_without_dot: HashSet<BareName>,
}

impl<'a> TypeResolver for Context<'a> {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.borrow().char_to_qualifier(ch)
    }
}

impl<'a> HasFunctions for Context<'a> {
    fn functions(&self) -> &FunctionMap {
        self.pre_linter_result.functions()
    }
}

impl<'a> HasSubs for Context<'a> {
    fn subs(&self) -> &SubMap {
        self.pre_linter_result.subs()
    }
}

impl<'a> HasUserDefinedTypes for Context<'a> {
    fn user_defined_types(&self) -> &UserDefinedTypes {
        self.pre_linter_result.user_defined_types()
    }
}

impl<'a> Context<'a> {
    pub fn new(
        pre_linter_result: &'a PreLinterResult,
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
    pub fn function_qualifier(&self, bare_name: &BareName) -> Option<TypeQualifier> {
        self.functions().get(bare_name).map(
            |Locatable {
                 element: (q, _), ..
             }| *q,
        )
    }
}
