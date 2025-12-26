use rusty_common::{Position, Positioned};
use rusty_parser::*;

use crate::LintErrorPos;

/// The result of a visitor.
pub type VisitResult = Result<(), LintErrorPos>;

/// A visitor can visit an immutable element by reference.
/// The visitor can mutate its own state.
pub trait Visitor<T> {
    /// Visits the given element.
    fn visit(&mut self, element: &T) -> VisitResult;
}

/// Indicates an object that can hold the most recently visited [Position].
pub trait SetPosition {
    /// Sets the most recently visited [Position].
    fn set_position(&mut self, pos: Position);
}

// Blanket implementation for Vec

impl<P, T> Visitor<Vec<T>> for P
where
    P: Visitor<T>,
{
    fn visit(&mut self, elements: &Vec<T>) -> VisitResult {
        for element in elements {
            self.visit(element)?;
        }
        Ok(())
    }
}

// Blanket implementation for Positioned

impl<P, T> Visitor<Positioned<T>> for P
where
    P: Visitor<T> + SetPosition,
{
    fn visit(&mut self, element: &Positioned<T>) -> VisitResult {
        let Positioned { element, pos } = element;
        self.set_position(*pos);
        self.visit(element)
    }
}

/// Indicates a visitor that can visit global statements
/// but does not enter the implementations of functions or subs.
pub trait ShallowGlobalStatementVisitor:
    Visitor<DefType>
    + Visitor<FunctionDeclaration>
    + Visitor<FunctionImplementation>
    + Visitor<Statement>
    + Visitor<SubDeclaration>
    + Visitor<SubImplementation>
    + Visitor<UserDefinedType>
{
}

impl<P> Visitor<GlobalStatement> for P
where
    P: ShallowGlobalStatementVisitor,
{
    fn visit(&mut self, element: &GlobalStatement) -> VisitResult {
        match element {
            GlobalStatement::DefType(def_type) => self.visit(def_type),
            GlobalStatement::FunctionDeclaration(f) => self.visit(f),
            GlobalStatement::FunctionImplementation(f) => self.visit(f),
            GlobalStatement::Statement(statement) => self.visit(statement),
            GlobalStatement::SubDeclaration(s) => self.visit(s),
            GlobalStatement::SubImplementation(s) => self.visit(s),
            GlobalStatement::UserDefinedType(user_defined_type) => self.visit(user_defined_type),
        }
    }
}

pub trait DelegateVisitor<T> {
    fn delegate(&mut self) -> impl Visitor<T>;
}
