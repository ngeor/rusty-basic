use crate::common::{CaseInsensitiveString, HasLocation, Location};
use crate::interpreter::{InterpreterError, Result};
use crate::parser::{
    BlockNode, HasQualifier, NameNode, QualifiedName, ResolveInto, ResolveIntoRef, TypeResolver,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct QualifiedDeclarationNode<T: Clone> {
    pub name: T,
    pub parameters: Vec<QualifiedName>,
    pub pos: Location,
}

impl<T: Clone> QualifiedDeclarationNode<T> {
    pub fn new<TR: TypeResolver, TName>(
        name: TName,
        parameters: Vec<NameNode>,
        pos: Location,
        resolver: &TR,
    ) -> Self
    where
        TName: ResolveInto<T>,
    {
        QualifiedDeclarationNode {
            name: name.resolve_into(resolver),
            parameters: parameters
                .into_iter()
                .map(|x| x.resolve_into(resolver))
                .collect(),
            pos: pos,
        }
    }
}

#[derive(Clone, Debug)]
pub struct QualifiedImplementationNode<T> {
    pub name: T,
    pub parameters: Vec<QualifiedName>,
    pub block: BlockNode,
    pub pos: Location,
}

impl<T: Clone> QualifiedImplementationNode<T> {
    pub fn new<TR: TypeResolver, TName>(
        name: TName,
        parameters: Vec<NameNode>,
        block: BlockNode,
        pos: Location,
        resolver: &TR,
    ) -> Self
    where
        TName: ResolveInto<T>,
    {
        QualifiedImplementationNode {
            name: name.resolve_into(resolver),
            parameters: parameters
                .into_iter()
                .map(|x| x.resolve_into(resolver))
                .collect(),
            block: block,
            pos: pos,
        }
    }
}

#[derive(Debug)]
pub struct SubprogramContext<T: Clone> {
    pub declarations: HashMap<CaseInsensitiveString, QualifiedDeclarationNode<T>>,
    pub implementations: HashMap<CaseInsensitiveString, QualifiedImplementationNode<T>>,
}

pub trait CmpQualifier<U> {
    fn eq_qualifier<TR: TypeResolver>(left: &Self, right: &U, resolver: &TR) -> bool;
}

impl<T: Clone> SubprogramContext<T> {
    pub fn new() -> Self {
        SubprogramContext {
            declarations: HashMap::new(),
            implementations: HashMap::new(),
        }
    }

    pub fn ensure_all_declared_programs_are_implemented(&self) -> Result<()> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return Err(InterpreterError::new_with_pos(
                    "Subprogram not defined",
                    v.pos,
                ));
            }
        }
        Ok(())
    }

    pub fn has_implementation(&self, name: &CaseInsensitiveString) -> bool {
        self.implementations.contains_key(name)
    }

    pub fn get_implementation(
        &self,
        name: &CaseInsensitiveString,
    ) -> Option<QualifiedImplementationNode<T>> {
        self.implementations.get(name).map(|x| x.clone())
    }

    pub fn add_declaration<TR: TypeResolver, TName>(
        &mut self,
        name: TName,
        parameters: Vec<NameNode>,
        pos: Location,
        resolver: &TR,
    ) -> Result<()>
    where
        TName: AsRef<CaseInsensitiveString> + CmpQualifier<T> + ResolveInto<T>,
    {
        match self.validate_against_existing_declaration(&name, &parameters, pos, resolver)? {
            None => {
                let bare_name: &CaseInsensitiveString = name.as_ref();
                self.declarations.insert(
                    bare_name.clone(),
                    QualifiedDeclarationNode::new(name, parameters, pos, resolver),
                );
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn add_implementation<TR: TypeResolver, TName>(
        &mut self,
        name: TName,
        parameters: Vec<NameNode>,
        block: BlockNode,
        pos: Location,
        resolver: &TR,
    ) -> Result<()>
    where
        TName: AsRef<CaseInsensitiveString> + CmpQualifier<T> + ResolveInto<T>,
    {
        let bare_name: &CaseInsensitiveString = name.as_ref();
        if self.has_implementation(bare_name) {
            Err(InterpreterError::new_with_pos("Duplicate definition", pos))
        } else {
            self.validate_against_existing_declaration(&name, &parameters, pos, resolver)?;
            self.implementations.insert(
                bare_name.clone(),
                QualifiedImplementationNode::new(name, parameters, block, pos, resolver),
            );
            Ok(())
        }
    }

    fn validate_against_existing_declaration<TR: TypeResolver, TName>(
        &self,
        name: &TName,
        parameters: &Vec<NameNode>,
        pos: Location,
        resolver: &TR,
    ) -> Result<Option<&QualifiedDeclarationNode<T>>>
    where
        TName: AsRef<CaseInsensitiveString> + CmpQualifier<T>,
    {
        let bare_name: &CaseInsensitiveString = name.as_ref();
        match self.declarations.get(bare_name) {
            Some(existing_declaration) => {
                if !CmpQualifier::eq_qualifier(name, &existing_declaration.name, resolver) {
                    Err(InterpreterError::new_with_pos("Duplicate definition", pos))
                } else {
                    require_parameters_same(
                        &existing_declaration.parameters,
                        &parameters,
                        pos,
                        resolver,
                    )?;
                    Ok(Some(existing_declaration))
                }
            }
            None => Ok(None),
        }
    }
}

fn require_parameters_same<T: TypeResolver>(
    existing: &Vec<QualifiedName>,
    parameters: &Vec<NameNode>,
    pos: Location,
    resolver: &T,
) -> Result<()> {
    if existing.len() != parameters.len() {
        return Err(InterpreterError::new_with_pos(
            "Argument-count mismatch",
            pos,
        ));
    }

    for i in 0..existing.len() {
        let e = &existing[i];
        let n = &parameters[i];
        if e.qualifier() != n.resolve_into(resolver) {
            return Err(InterpreterError::new_with_pos(
                "Parameter type mismatch",
                n.location(),
            ));
        }
    }

    Ok(())
}
