use crate::core::{IntoTypeQualifier, TypeResolver};
use crate::core::{LintError, LintErrorPos};
use crate::pre_linter::{FunctionSignature, ResolvedParamTypes, SubSignature};
use rusty_common::*;
use rusty_parser::specific::{BareName, Name};
use rusty_parser::BuiltInFunction;
use rusty_parser::BuiltInSub;
use std::collections::HashMap;

pub struct SubprogramContext<T> {
    declarations: HashMap<CaseInsensitiveString, Positioned<T>>,
    implementations: HashMap<CaseInsensitiveString, Positioned<T>>,
}

pub type FunctionContext = SubprogramContext<FunctionSignature>;
pub type SubContext = SubprogramContext<SubSignature>;

trait CheckSignature<T>
where
    T: PartialEq,
{
    /// Checks the signature of the given subprogram name against already known definitions.
    /// Returns an error if the signature doesn't match.
    /// Returns true if the definition already exists.
    /// Returns false if the definition doesn't exist.
    fn check_signature(&self, name: &BareName, signature: &T) -> Result<bool, LintError>;
}

impl<T> CheckSignature<T> for HashMap<CaseInsensitiveString, Positioned<T>>
where
    T: PartialEq,
{
    fn check_signature(&self, name: &BareName, signature: &T) -> Result<bool, LintError> {
        if let Some(Positioned { element, .. }) = self.get(name) {
            if element != signature {
                Err(LintError::TypeMismatch)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

/// Converts a sub-program name into a sub-program signature.
pub trait ToSignature {
    type Signature;

    fn to_signature(
        &self,
        resolver: &impl TypeResolver,
        qualified_params: ResolvedParamTypes,
    ) -> Self::Signature;
}

impl ToSignature for BareName {
    type Signature = SubSignature;

    fn to_signature(
        &self,
        _resolver: &impl TypeResolver,
        qualified_params: ResolvedParamTypes,
    ) -> Self::Signature {
        SubSignature::new(qualified_params)
    }
}

impl ToSignature for Name {
    type Signature = FunctionSignature;

    fn to_signature(
        &self,
        resolver: &impl TypeResolver,
        qualified_params: ResolvedParamTypes,
    ) -> Self::Signature {
        let q = self.qualify(resolver);
        FunctionSignature::new(q, qualified_params)
    }
}

impl<T> SubprogramContext<T>
where
    T: PartialEq,
{
    pub fn new() -> Self {
        Self {
            declarations: HashMap::new(),
            implementations: HashMap::new(),
        }
    }

    pub fn add_declaration(
        &mut self,
        bare_name: &BareName,
        signature: T,
        declaration_pos: Position,
    ) -> Result<(), LintError> {
        self.implementations
            .check_signature(bare_name, &signature)?;
        if !self.declarations.check_signature(bare_name, &signature)? {
            self.declarations
                .insert(bare_name.clone(), signature.at_pos(declaration_pos));
        }
        Ok(())
    }

    pub fn add_implementation(
        &mut self,
        bare_name: &BareName,
        signature: T,
        implementation_pos: Position,
    ) -> Result<(), LintError> {
        match self.implementations.get(bare_name) {
            Some(_) => Err(LintError::DuplicateDefinition),
            None => {
                self.declarations.check_signature(bare_name, &signature)?;
                self.implementations
                    .insert(bare_name.clone(), signature.at_pos(implementation_pos));
                Ok(())
            }
        }
    }

    pub fn implementations(self) -> HashMap<CaseInsensitiveString, Positioned<T>> {
        self.implementations
    }

    fn ensure_declarations_are_implemented(&self) -> Result<(), LintErrorPos> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return Err(LintError::SubprogramNotDefined.at(v));
            }
        }
        Ok(())
    }

    fn ensure_does_not_clash_with_built_in<F>(&self, is_built_in: F) -> Result<(), LintErrorPos>
    where
        F: Fn(&BareName) -> bool,
    {
        for (k, v) in self.implementations.iter() {
            if is_built_in(k) {
                return Err(LintError::DuplicateDefinition.at(v));
            }
        }

        Ok(())
    }
}

impl FunctionContext {
    pub fn post_visit(&self) -> Result<(), LintErrorPos> {
        self.ensure_declarations_are_implemented()?;
        self.ensure_does_not_clash_with_built_in(|name| BuiltInFunction::try_parse(name).is_some())
    }
}

impl SubContext {
    pub fn post_visit(&self) -> Result<(), LintErrorPos> {
        // not checking if declarations are present, because in MONEY.BAS there
        // are two SUBs declared but not implemented (and not called either)
        self.ensure_does_not_clash_with_built_in(|name| {
            BuiltInSub::parse_non_keyword_sub(name.as_ref()).is_some()
        })
    }
}
