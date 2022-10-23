use crate::common::*;
use crate::linter::converter::names::Names;
use crate::linter::converter::traits::Convertible;
use crate::linter::pre_linter::PreLinterResult;
use crate::linter::type_resolver::{IntoQualified, TypeResolver};
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::{FunctionMap, HasFunctions, HasSubs, HasUserDefinedTypes, NameContext, SubMap};
use crate::parser::*;
use std::collections::HashSet;
use std::rc::Rc;

pub fn convert(
    program: ProgramNode,
    pre_linter_result: Rc<PreLinterResult>,
) -> Result<(ProgramNode, HashSet<BareName>), QErrorNode> {
    let mut converter = Context::new(pre_linter_result);
    let result = converter.convert_program(program)?;
    // consume
    let names_without_dot = converter.consume();
    Ok((result, names_without_dot))
}

/// Alias for the implicit variables collected during evaluating something.
/// e.g. `INPUT N` is a statement implicitly defining variable `N`.
pub type Implicits = Vec<QualifiedNameNode>;

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
    resolver: TypeResolverImpl,
    pub names: Names,
    names_without_dot: HashSet<BareName>,
}

impl TypeResolver for Context {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.char_to_qualifier(ch)
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
    pub fn new(pre_linter_result: Rc<PreLinterResult>) -> Self {
        Self {
            pre_linter_result,
            resolver: TypeResolverImpl::new(),
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
        params.convert(self)
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
        Ok((converted_function_name, params.convert(self)?))
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

    /// Gets the function qualifier of the function identified by the given bare name.
    /// If no such function exists, returns `None`.
    pub fn function_qualifier(&self, bare_name: &BareName) -> Option<TypeQualifier> {
        self.functions()
            .get(bare_name)
            .map(|function_signature_node| function_signature_node.as_ref().qualifier())
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
        let mut result = statements.convert(self)?;
        let implicits = self.pop_context();
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

    pub fn consume(self) -> HashSet<BareName> {
        self.names_without_dot()
    }

    pub fn convert_program(&mut self, program: ProgramNode) -> Result<ProgramNode, QErrorNode> {
        let mut result: ProgramNode = vec![];
        for Locatable { element, pos } in program {
            match element {
                TopLevelToken::DefType(def_type) => {
                    self.resolver.set(&def_type);
                }
                TopLevelToken::FunctionDeclaration(_name, _params) => {}
                TopLevelToken::FunctionImplementation(f) => {
                    let converted = self.convert_function_implementation(f)?;
                    result.push(converted.at(pos));
                }
                TopLevelToken::Statement(s) => {
                    let converted = vec![s.at(pos)].convert(self)?;
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
        implicits.append(self.names.get_implicits());
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
