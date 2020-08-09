use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::linter::type_resolver::*;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser;
use crate::parser::{
    BareName, DeclaredName, DeclaredNameNode, DeclaredNameNodes, NameNode, TypeDefinition,
    TypeQualifier,
};
use std::collections::HashMap;

/// Collects subprograms of the given program.
/// Ensures that:
/// - All declared subprograms are implemented
/// - No duplicate implementations
/// - No conflicts between declarations and implementations
/// - Resolves types of parameters and functions
pub fn collect_subprograms(p: &parser::ProgramNode) -> Result<(FunctionMap, SubMap), QErrorNode> {
    let mut f_c = FunctionContext::new();
    let mut s_c = SubContext::new();
    for t in p {
        f_c.visit(t)?;
        s_c.visit(t)?;
    }
    f_c.post_visit()?;
    s_c.post_visit()?;
    Ok((f_c.implementations, s_c.implementations))
}

pub type ParamTypes = Vec<TypeQualifier>;
pub type FunctionMap = HashMap<CaseInsensitiveString, (TypeQualifier, ParamTypes, Location)>;

#[derive(Debug, Default)]
struct FunctionContext {
    resolver: TypeResolverImpl,
    declarations: FunctionMap,
    implementations: FunctionMap,
}

fn resolve_declared_name_node<T: TypeResolver>(
    resolver: &T,
    p: &DeclaredNameNode,
) -> TypeQualifier {
    let d: &DeclaredName = p.as_ref();
    match d.type_definition() {
        TypeDefinition::Bare => {
            let bare_name: &BareName = d.as_ref();
            bare_name.resolve_into(resolver)
        }
        TypeDefinition::CompactBuiltIn(q) | TypeDefinition::ExtendedBuiltIn(q) => *q,
        _ => unimplemented!(),
    }
}

impl FunctionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_declaration(
        &mut self,
        name: &NameNode,
        params: &DeclaredNameNodes,
        pos: Location,
    ) -> Result<(), QErrorNode> {
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let q_params: Vec<TypeQualifier> = params
            .iter()
            .map(|p| resolve_declared_name_node(&self.resolver, p))
            .collect();
        let q_name: TypeQualifier = name.resolve_into(&self.resolver);
        let bare_name = BareName::from(name);
        self.check_implementation_type(&bare_name, &q_name, &q_params, pos)?;
        match self.declarations.get(&bare_name) {
            Some(_) => self.check_declaration_type(&bare_name, &q_name, &q_params, pos),
            None => {
                self.declarations.insert(bare_name, (q_name, q_params, pos));
                Ok(())
            }
        }
    }

    pub fn add_implementation(
        &mut self,
        name: &NameNode,
        params: &DeclaredNameNodes,
        pos: Location,
    ) -> Result<(), QErrorNode> {
        // type must match declaration
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let q_params: Vec<TypeQualifier> = params
            .iter()
            .map(|p| resolve_declared_name_node(&self.resolver, p))
            .collect();
        let q_name: TypeQualifier = name.resolve_into(&self.resolver);
        let bare_name = BareName::from(name);
        match self.implementations.get(&bare_name) {
            Some(_) => Err(QError::DuplicateDefinition).with_err_at(pos),
            None => {
                self.check_declaration_type(&bare_name, &q_name, &q_params, pos)?;
                self.implementations
                    .insert(bare_name, (q_name, q_params, pos));
                Ok(())
            }
        }
    }

    fn check_declaration_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_name: &TypeQualifier,
        q_params: &Vec<TypeQualifier>,
        pos: Location,
    ) -> Result<(), QErrorNode> {
        match self.declarations.get(name) {
            Some((e_name, e_params, _)) => {
                if e_name != q_name || e_params != q_params {
                    return Err(QError::TypeMismatch).with_err_at(pos);
                }
            }
            None => (),
        }
        Ok(())
    }

    fn check_implementation_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_name: &TypeQualifier,
        q_params: &Vec<TypeQualifier>,
        pos: Location,
    ) -> Result<(), QErrorNode> {
        match self.implementations.get(name) {
            Some((e_name, e_params, _)) => {
                if e_name != q_name || e_params != q_params {
                    return Err(QError::TypeMismatch).with_err_at(pos);
                }
            }
            None => (),
        }
        Ok(())
    }

    pub fn visit(&mut self, a: &parser::TopLevelTokenNode) -> Result<(), QErrorNode> {
        let pos = a.pos();
        match a.as_ref() {
            parser::TopLevelToken::DefType(d) => {
                self.resolver.set(d);
                Ok(())
            }
            parser::TopLevelToken::FunctionDeclaration(n, params) => {
                self.add_declaration(n, params, pos)
            }
            parser::TopLevelToken::FunctionImplementation(n, params, _) => {
                self.add_implementation(n, params, pos)
            }
            _ => Ok(()),
        }
    }

    pub fn post_visit(&mut self) -> Result<(), QErrorNode> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return Err(QError::SubprogramNotDefined).with_err_at(v.2);
            }
        }

        for (k, v) in self.implementations.iter() {
            let opt_built_in: Option<BuiltInFunction> = k.into();
            if opt_built_in.is_some() {
                return Err(QError::DuplicateDefinition).with_err_at(v.2);
            }
        }

        Ok(())
    }
}

pub type SubMap = HashMap<CaseInsensitiveString, (ParamTypes, Location)>;

#[derive(Debug, Default)]
struct SubContext {
    resolver: TypeResolverImpl,
    declarations: SubMap,
    implementations: SubMap,
    errors: Vec<Locatable<String>>,
}

impl SubContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_declaration(
        &mut self,
        name: &CaseInsensitiveString,
        params: &DeclaredNameNodes,
        pos: Location,
    ) -> Result<(), QErrorNode> {
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let q_params: Vec<TypeQualifier> = params
            .iter()
            .map(|p| resolve_declared_name_node(&self.resolver, p))
            .collect();
        self.check_implementation_type(name, &q_params, pos)?;
        match self.declarations.get(name) {
            Some(_) => self.check_declaration_type(name, &q_params, pos),
            None => {
                self.declarations.insert(name.clone(), (q_params, pos));
                Ok(())
            }
        }
    }

    pub fn add_implementation(
        &mut self,
        name: &CaseInsensitiveString,
        params: &DeclaredNameNodes,
        pos: Location,
    ) -> Result<(), QErrorNode> {
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let q_params: Vec<TypeQualifier> = params
            .iter()
            .map(|p| resolve_declared_name_node(&self.resolver, p))
            .collect();
        match self.implementations.get(name) {
            Some(_) => Err(QError::DuplicateDefinition).with_err_at(pos),
            None => {
                self.check_declaration_type(name, &q_params, pos)?;
                self.implementations.insert(name.clone(), (q_params, pos));
                Ok(())
            }
        }
    }

    fn check_declaration_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_params: &Vec<TypeQualifier>,
        pos: Location,
    ) -> Result<(), QErrorNode> {
        match self.declarations.get(name) {
            Some((e_params, _)) => {
                if e_params != q_params {
                    return Err(QError::TypeMismatch).with_err_at(pos);
                }
            }
            None => (),
        }
        Ok(())
    }

    fn check_implementation_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_params: &Vec<TypeQualifier>,
        pos: Location,
    ) -> Result<(), QErrorNode> {
        match self.implementations.get(name) {
            Some((e_params, _)) => {
                if e_params != q_params {
                    return Err(QError::TypeMismatch).with_err_at(pos);
                }
            }
            None => (),
        }
        Ok(())
    }

    pub fn visit(
        &mut self,
        top_level_token_node: &parser::TopLevelTokenNode,
    ) -> Result<(), QErrorNode> {
        let Locatable {
            element: top_level_token,
            pos,
        } = top_level_token_node;
        match top_level_token {
            parser::TopLevelToken::DefType(d) => {
                self.resolver.set(d);
                Ok(())
            }
            parser::TopLevelToken::SubDeclaration(n, params) => {
                self.add_declaration(n.as_ref(), params, *pos)
            }
            parser::TopLevelToken::SubImplementation(n, params, _) => {
                self.add_implementation(n.as_ref(), params, *pos)
            }
            _ => Ok(()),
        }
    }

    pub fn post_visit(&mut self) -> Result<(), QErrorNode> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return Err(QError::SubprogramNotDefined).with_err_at(v.1);
            }
        }

        for (k, v) in self.implementations.iter() {
            let opt_built_in: Option<BuiltInSub> = k.into();
            if opt_built_in.is_some() {
                return Err(QError::DuplicateDefinition).with_err_at(v.1);
            }
        }

        Ok(())
    }
}
