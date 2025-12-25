use rusty_common::{Position, Positioned};
use rusty_parser::*;

use crate::LintErrorPos;

pub type VisitResult = Result<(), LintErrorPos>;

pub trait Visitor<T> {
    fn visit(&mut self, element: &T) -> VisitResult;
}

pub trait PosVisitor {
    fn set_pos(&mut self, pos: Position);
}

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

impl<P, T> Visitor<Positioned<T>> for P
where
    P: Visitor<T> + PosVisitor,
{
    fn visit(&mut self, element: &Positioned<T>) -> VisitResult {
        let Positioned { element, pos } = element;
        self.set_pos(*pos);
        self.visit(element)
    }
}

impl<P> Visitor<GlobalStatement> for P
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
