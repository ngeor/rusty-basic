use crate::core::*;
use rusty_common::*;
use rusty_parser::BareName;

#[derive(Default)]
pub struct SubprogramContext {
    declarations: Declarations,
    implementations: Implementations,
}

#[derive(Default)]
struct Declarations(SignatureMap);

#[derive(Default)]
struct Implementations(SignatureMap);

trait CheckSignature {
    /// Checks the signature of the given subprogram name against already known definitions.
    /// Returns an error if the signature doesn't match.
    /// Returns true if the definition already exists.
    /// Returns false if the definition doesn't exist.
    fn check_signature(
        &self,
        name: &BareName,
        signature: &Positioned<Signature>,
    ) -> Result<bool, LintErrorPos>;
}

impl CheckSignature for SignatureMap {
    fn check_signature(
        &self,
        name: &BareName,
        signature: &Positioned<Signature>,
    ) -> Result<bool, LintErrorPos> {
        if let Some(Positioned { element, .. }) = self.get(name) {
            if element != &signature.element {
                Err(LintError::TypeMismatch.at_pos(signature.pos))
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

impl CheckSignature for Declarations {
    fn check_signature(
        &self,
        name: &BareName,
        signature: &Positioned<Signature>,
    ) -> Result<bool, LintErrorPos> {
        self.0.check_signature(name, signature)
    }
}

impl CheckSignature for Implementations {
    fn check_signature(
        &self,
        name: &BareName,
        signature: &Positioned<Signature>,
    ) -> Result<bool, LintErrorPos> {
        self.0.check_signature(name, signature)
    }
}

impl Declarations {
    pub fn insert(
        &mut self,
        bare_name: BareName,
        signature: Positioned<Signature>,
    ) -> Result<(), LintErrorPos> {
        // add if it doesn't already exist, tolerate multiple declarations as long as the signature matches
        self.check_signature(&bare_name, &signature).map(|exists| {
            if !exists {
                self.0.insert(bare_name, signature);
            }
            ()
        })
    }

    fn iter(&self) -> impl Iterator<Item = (&BareName, &Positioned<Signature>)> {
        self.0.iter()
    }
}

impl Implementations {
    pub fn insert(
        &mut self,
        bare_name: BareName,
        signature: Positioned<Signature>,
    ) -> Result<(), LintErrorPos> {
        // add if doesn't already exist, do not tolerate multiple implementations
        if self.0.contains_key(&bare_name) {
            Err(LintError::DuplicateDefinition.at_pos(signature.pos))
        } else {
            self.0.insert(bare_name, signature);
            Ok(())
        }
    }

    fn iter(&self) -> impl Iterator<Item = (&BareName, &Positioned<Signature>)> {
        self.0.iter()
    }

    fn contains_key(&self, name: &BareName) -> bool {
        self.0.contains_key(name)
    }
}

impl SubprogramContext {
    pub fn add_declaration(
        &mut self,
        bare_name: BareName,
        signature: Positioned<Signature>,
    ) -> Result<(), LintErrorPos> {
        self.implementations
            .check_signature(&bare_name, &signature)?;
        self.declarations.insert(bare_name, signature)
    }

    pub fn add_implementation(
        &mut self,
        bare_name: BareName,
        signature: Positioned<Signature>,
    ) -> Result<(), LintErrorPos> {
        self.declarations.check_signature(&bare_name, &signature)?;
        self.implementations.insert(bare_name, signature)
    }

    pub fn ensure_declarations_are_implemented(&self) -> Result<(), LintErrorPos> {
        for (name, signature) in self.declarations.iter() {
            if !self.implementations.contains_key(name) {
                return Err(LintError::SubprogramNotDefined.at(signature));
            }
        }
        Ok(())
    }

    /// Ensures the collected names do not clash with built-in names (e.g. `BEEP`, `LEN`, etc).
    /// The [is_built_in] predicate tests if a name is a built-in name or not.
    pub fn ensure_does_not_clash_with_built_in<F>(&self, is_built_in: F) -> Result<(), LintErrorPos>
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

    pub fn implementations(self) -> SignatureMap {
        self.implementations.0
    }
}
