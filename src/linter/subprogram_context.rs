use super::built_in_function_linter::is_built_in_function;
use super::built_in_sub_linter::is_built_in_sub;
use super::error::*;
use crate::common::*;
use crate::parser;
use crate::parser::type_resolver_impl::TypeResolverImpl;
use crate::parser::{NameNode, NameTrait, TypeQualifier, TypeResolver};
use std::collections::HashMap;

//
// Visitor trait
//

/// A visitor visits an object. It might update itself on each visit.
trait Visitor<A> {
    fn visit(&mut self, a: &A) -> Result<(), Error>;
}

trait PostVisitor<A> {
    fn post_visit(&mut self, a: &A) -> Result<(), Error>;
}

/// Blanket visitor implementation for vectors.
impl<T, A> Visitor<Vec<A>> for T
where
    T: Visitor<A> + PostVisitor<Vec<A>>,
{
    fn visit(&mut self, a: &Vec<A>) -> Result<(), Error> {
        for x in a.iter() {
            self.visit(x)?;
        }
        self.post_visit(a)
    }
}

pub type ParamTypes = Vec<TypeQualifier>;
pub type FunctionMap = HashMap<CaseInsensitiveString, (TypeQualifier, ParamTypes, Location)>;

#[derive(Debug, Default)]
struct FunctionContext {
    resolver: TypeResolverImpl,
    declarations: FunctionMap,
    implementations: FunctionMap,
}

impl FunctionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_declaration(
        &mut self,
        name: &NameNode,
        params: &Vec<NameNode>,
        pos: Location,
    ) -> Result<(), Error> {
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let q_params: Vec<TypeQualifier> =
            params.iter().map(|p| self.resolver.resolve(p)).collect();
        let q_name: TypeQualifier = self.resolver.resolve(name);
        let bare_name = name.bare_name().clone();
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
        params: &Vec<NameNode>,
        pos: Location,
    ) -> Result<(), Error> {
        // type must match declaration
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let q_params: Vec<TypeQualifier> =
            params.iter().map(|p| self.resolver.resolve(p)).collect();
        let q_name: TypeQualifier = self.resolver.resolve(name);
        let bare_name = name.bare_name().clone();
        match self.implementations.get(&bare_name) {
            Some(_) => err(LinterError::DuplicateDefinition, pos),
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
    ) -> Result<(), Error> {
        match self.declarations.get(name) {
            Some((e_name, e_params, _)) => {
                if e_name != q_name || e_params != q_params {
                    return err(LinterError::TypeMismatch, pos);
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
    ) -> Result<(), Error> {
        match self.implementations.get(name) {
            Some((e_name, e_params, _)) => {
                if e_name != q_name || e_params != q_params {
                    return err(LinterError::TypeMismatch, pos);
                }
            }
            None => (),
        }
        Ok(())
    }
}

impl Visitor<parser::TopLevelTokenNode> for FunctionContext {
    fn visit(&mut self, a: &parser::TopLevelTokenNode) -> Result<(), Error> {
        let pos = a.location();
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
}

impl PostVisitor<parser::ProgramNode> for FunctionContext {
    fn post_visit(&mut self, _: &parser::ProgramNode) -> Result<(), Error> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return err(LinterError::SubprogramNotDefined, v.2);
            }
        }

        for (k, v) in self.implementations.iter() {
            if is_built_in_function(k) {
                return err(LinterError::DuplicateDefinition, v.2);
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
        params: &Vec<NameNode>,
        pos: Location,
    ) -> Result<(), Error> {
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let q_params: Vec<TypeQualifier> =
            params.iter().map(|p| self.resolver.resolve(p)).collect();
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
        params: &Vec<NameNode>,
        pos: Location,
    ) -> Result<(), Error> {
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let q_params: Vec<TypeQualifier> =
            params.iter().map(|p| self.resolver.resolve(p)).collect();
        match self.implementations.get(name) {
            Some(_) => err(LinterError::DuplicateDefinition, pos),
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
    ) -> Result<(), Error> {
        match self.declarations.get(name) {
            Some((e_params, _)) => {
                if e_params != q_params {
                    return err(LinterError::TypeMismatch, pos);
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
    ) -> Result<(), Error> {
        match self.implementations.get(name) {
            Some((e_params, _)) => {
                if e_params != q_params {
                    return err(LinterError::TypeMismatch, pos);
                }
            }
            None => (),
        }
        Ok(())
    }
}

impl Visitor<parser::TopLevelTokenNode> for SubContext {
    fn visit(&mut self, a: &parser::TopLevelTokenNode) -> Result<(), Error> {
        let pos = a.location();
        match a.as_ref() {
            parser::TopLevelToken::DefType(d) => {
                self.resolver.set(d);
                Ok(())
            }
            parser::TopLevelToken::SubDeclaration(n, params) => {
                self.add_declaration(n.as_ref(), params, pos)
            }
            parser::TopLevelToken::SubImplementation(n, params, _) => {
                self.add_implementation(n.as_ref(), params, pos)
            }
            _ => Ok(()),
        }
    }
}

impl PostVisitor<parser::ProgramNode> for SubContext {
    fn post_visit(&mut self, _: &parser::ProgramNode) -> Result<(), Error> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return err(LinterError::SubprogramNotDefined, v.1);
            }
        }

        for (k, v) in self.implementations.iter() {
            if is_built_in_sub(k) {
                return err(LinterError::DuplicateDefinition, v.1);
            }
        }

        Ok(())
    }
}

/// Collects subprograms of the given program.
/// Ensures that:
/// - All declared subprograms are implemented
/// - No duplicate implementations
/// - No conflicts between declarations and implementations
/// - Resolves types of parameters and functions
pub fn collect_subprograms(p: &parser::ProgramNode) -> Result<(FunctionMap, SubMap), Error> {
    let mut f_c = FunctionContext::new();
    f_c.visit(p)?;
    let mut s_c = SubContext::new();
    s_c.visit(p)?;
    Ok((f_c.implementations, s_c.implementations))
}
