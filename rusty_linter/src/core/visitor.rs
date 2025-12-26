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

/// Creates a delegate visitor for the given type T.
pub trait DelegateVisitor<T> {
    fn delegate(&mut self) -> impl Visitor<T>;
}

/// A visitor that can visit global statements
/// but does not enter the implementations of functions or subs.
/// Actual visiting logic is handled by the delegate.
pub struct ShallowVisitor<P> {
    delegate: P,
}

impl<P> ShallowVisitor<P> {
    pub fn new(delegate: P) -> Self {
        Self { delegate }
    }

    /// Returns the delegate back.
    pub fn delegate(self) -> P {
        self.delegate
    }
}

impl<P> SetPosition for ShallowVisitor<P>
where
    P: SetPosition,
{
    fn set_position(&mut self, pos: Position) {
        self.delegate.set_position(pos);
    }
}

impl<P> Visitor<GlobalStatement> for ShallowVisitor<P>
where
    P: Visitor<DefType>
        + Visitor<FunctionDeclaration>
        + Visitor<FunctionImplementation>
        + Visitor<Statement>
        + Visitor<SubDeclaration>
        + Visitor<SubImplementation>
        + Visitor<UserDefinedType>,
{
    fn visit(&mut self, element: &GlobalStatement) -> VisitResult {
        match element {
            GlobalStatement::DefType(def_type) => self.delegate.visit(def_type),
            GlobalStatement::FunctionDeclaration(f) => self.delegate.visit(f),
            GlobalStatement::FunctionImplementation(f) => self.delegate.visit(f),
            GlobalStatement::Statement(statement) => self.delegate.visit(statement),
            GlobalStatement::SubDeclaration(s) => self.delegate.visit(s),
            GlobalStatement::SubImplementation(s) => self.delegate.visit(s),
            GlobalStatement::UserDefinedType(user_defined_type) => {
                self.delegate.visit(user_defined_type)
            }
        }
    }
}
