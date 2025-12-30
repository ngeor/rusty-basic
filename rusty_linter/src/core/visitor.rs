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

// Blanket implementation for Option

impl<P, T> Visitor<Option<T>> for P
where
    P: Visitor<T>,
{
    fn visit(&mut self, element: &Option<T>) -> VisitResult {
        match element {
            Some(t) => self.visit(t),
            _ => Ok(()),
        }
    }
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

/// A visitor that delegates some functionality to another visitor.
pub trait DelegateVisitor<P> {
    /// Returns the delegate visitor.
    /// Typically called after the visit is done, in order to extract
    /// information from the underlying visitor.
    fn delegate(self) -> P;
}

delegate_visitor!(
    /// A visitor that can visit global statements
    /// but does not enter the implementations of functions or subs.
    /// Actual visiting logic is handled by the delegate.
    GlobalVisitor
);

impl<P> Visitor<GlobalStatement> for GlobalVisitor<P>
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

delegate_visitor!(
    /// A visitor that can visit statements
    /// and also enters the implementations of functions or subs,
    /// as well as nested block statements (e.g. inside IF statements).
    /// Actual visiting logic is handled by the delegate.
    DeepStatementVisitor
);

impl<P> Visitor<GlobalStatement> for DeepStatementVisitor<P>
where
    P: Visitor<DefType>
        + Visitor<FunctionDeclaration>
        + Visitor<FunctionImplementation>
        + Visitor<SubDeclaration>
        + Visitor<SubImplementation>
        + Visitor<UserDefinedType>
        + Visitor<Statement>
        + SetPosition,
{
    fn visit(&mut self, element: &GlobalStatement) -> VisitResult {
        match element {
            GlobalStatement::DefType(def_type) => self.delegate.visit(def_type),
            GlobalStatement::FunctionDeclaration(f) => self.delegate.visit(f),
            GlobalStatement::FunctionImplementation(f) => {
                // notify first the delegate about the FUNCTION implementation
                self.delegate.visit(f)?;
                // then visit the body (this will go into the statements of the FUNCTION)
                self.visit(&f.body)
            }
            GlobalStatement::Statement(statement) => self.visit(statement),
            GlobalStatement::SubDeclaration(s) => self.delegate.visit(s),
            GlobalStatement::SubImplementation(s) => {
                // notify first the delegate about the SUB implementation
                self.delegate.visit(s)?;
                // then visit the body (this will go into the statements of the SUB)
                self.visit(&s.body)
            }
            GlobalStatement::UserDefinedType(user_defined_type) => {
                self.delegate.visit(user_defined_type)
            }
        }
    }
}

impl<P> Visitor<Statement> for DeepStatementVisitor<P>
where
    P: Visitor<Statement> + SetPosition,
{
    fn visit(&mut self, element: &Statement) -> VisitResult {
        // first visit the delegate
        self.delegate.visit(element)?;

        // then dive into the statement to for cases of blocks
        match element {
            Statement::IfBlock(if_block) => self.visit(if_block),
            Statement::SelectCase(select_case) => self.visit(select_case),
            Statement::ForLoop(for_loop) => self.visit(for_loop),
            Statement::While(conditional_block) => self.visit(conditional_block),
            Statement::DoLoop(do_loop) => self.visit(do_loop),
            _ => Ok(()),
        }
    }
}

impl<P> Visitor<ForLoop> for DeepStatementVisitor<P>
where
    P: Visitor<Statement> + SetPosition,
{
    fn visit(&mut self, element: &ForLoop) -> VisitResult {
        self.visit(&element.statements)
    }
}

impl<P> Visitor<SelectCase> for DeepStatementVisitor<P>
where
    P: Visitor<Statement> + SetPosition,
{
    fn visit(&mut self, element: &SelectCase) -> VisitResult {
        self.visit(&element.case_blocks)?;
        self.visit(&element.else_block)
    }
}

impl<P> Visitor<CaseBlock> for DeepStatementVisitor<P>
where
    P: Visitor<Statement> + SetPosition,
{
    fn visit(&mut self, element: &CaseBlock) -> VisitResult {
        let (_, statements) = element.into();
        self.visit(statements)
    }
}

impl<P> Visitor<IfBlock> for DeepStatementVisitor<P>
where
    P: Visitor<Statement> + SetPosition,
{
    fn visit(&mut self, element: &IfBlock) -> VisitResult {
        self.visit(&element.if_block)?;
        self.visit(&element.else_if_blocks)?;
        self.visit(&element.else_block)
    }
}

impl<P> Visitor<DoLoop> for DeepStatementVisitor<P>
where
    P: Visitor<Statement> + SetPosition,
{
    fn visit(&mut self, element: &DoLoop) -> VisitResult {
        self.visit(&element.statements)
    }
}

impl<P> Visitor<ConditionalBlock> for DeepStatementVisitor<P>
where
    P: Visitor<Statement> + SetPosition,
{
    fn visit(&mut self, element: &ConditionalBlock) -> VisitResult {
        self.visit(&element.statements)
    }
}

/// Creates a no-op visitor implementation
/// for the given types.
macro_rules! no_op_visitor {
    ($visitor_name: ident: $($types:tt),+) => {
        $(
            impl Visitor<$types> for $visitor_name {
                fn visit(&mut self, _element: &$types) -> VisitResult {
                    Ok(())
                }
            }
        )+
    };
}

macro_rules! no_pos_visitor {
    ($visitor_name: ident) => {
        impl SetPosition for $visitor_name {
            fn set_position(&mut self, _pos: Position) {}
        }
    };
}

/// Creates a visitor that delegates to another.
macro_rules! delegate_visitor {
    ($(#[$($attrss:tt)*])* $name: ident) => {
        $(#[$($attrss)*])*
        pub struct $name<P> {
            delegate: P
        }

        impl<P> $name<P> {
            pub fn new(delegate: P) -> Self {
                Self { delegate }
            }
        }

        impl<P> SetPosition for $name<P>
        where
            P: SetPosition,
        {
            fn set_position(&mut self, pos: Position) {
                self.delegate.set_position(pos);
            }
        }

        impl<P> DelegateVisitor<P> for $name<P> {
            fn delegate(self) -> P {
                self.delegate
            }
        }
    };
}

pub(crate) use {delegate_visitor, no_op_visitor, no_pos_visitor};
