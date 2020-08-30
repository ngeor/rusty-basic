use super::{parse_main_file, parse_main_str};
use crate::common::*;

use crate::parser::types::*;
use std::fs::File;

/// Parses the given program and demands success.
pub fn parse<T: AsRef<[u8]> + 'static>(input: T) -> ProgramNode {
    parse_main_str(input).expect("Could not parse program")
}

pub fn parse_file<S: AsRef<str>>(filename: S) -> ProgramNode {
    let file_path = format!("fixtures/{}", filename.as_ref());
    let f = File::open(file_path).expect("Could not read bas file");
    parse_main_file(f).expect("Could not parse program")
}

/// Parses the given input, expecting that it will fail.
/// Returns the lexer error.
/// Panics if parsing actually succeeded.
pub fn parse_err<T: AsRef<[u8]> + 'static>(input: T) -> QError {
    parse_main_str(input).unwrap_err().into_err()
}

pub trait ProgramNodeHelper {
    /// Parses the given input and asserts that it is parsed successfully and that
    /// it contains a single top level token node.
    ///
    /// Return the single top level token node of the parsed program.
    fn demand_single(self) -> TopLevelTokenNode;

    fn demand_single_statement(self) -> Statement;
}

impl ProgramNodeHelper for ProgramNode {
    fn demand_single(mut self) -> TopLevelTokenNode {
        assert_eq!(1, self.len());
        self.pop().unwrap()
    }

    fn demand_single_statement(self) -> Statement {
        let t = self.demand_single();
        match t {
            Locatable {
                element: TopLevelToken::Statement(s),
                ..
            } => s,
            _ => panic!(format!("Expected: statement, found {:?}", t)),
        }
    }
}

//
// Create NameNode out of literals
//

pub trait NameNodeFactory {
    fn as_name(&self, row: u32, col: u32) -> NameNode;
    fn as_bare_name(&self, row: u32, col: u32) -> BareNameNode;
}

impl NameNodeFactory for str {
    fn as_name(&self, row: u32, col: u32) -> NameNode {
        Name::from(self).at(Location::new(row, col))
    }

    fn as_bare_name(&self, row: u32, col: u32) -> BareNameNode {
        BareNameNode::new(
            CaseInsensitiveString::new(self.to_string()),
            Location::new(row, col),
        )
    }
}

//
// Create ExpressionNode out of literals
//

pub trait ExpressionNodeLiteralFactory {
    /// Creates an expression node holding a literal.
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionNode;
}

impl ExpressionNodeLiteralFactory for &str {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionNode {
        Expression::StringLiteral(self.to_string()).at_rc(row, col)
    }
}

impl ExpressionNodeLiteralFactory for i32 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionNode {
        Expression::IntegerLiteral(*self).at_rc(row, col)
    }
}

impl ExpressionNodeLiteralFactory for f32 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionNode {
        Expression::SingleLiteral(*self).at_rc(row, col)
    }
}

impl ExpressionNodeLiteralFactory for f64 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionNode {
        Expression::DoubleLiteral(*self).at_rc(row, col)
    }
}

//
// Create ExpressionNode out of literals
//

pub trait ExpressionNodeVariableFactory {
    /// Creates an expression node holding a variable name.
    fn as_var_expr(&self, row: u32, col: u32) -> ExpressionNode;
}

impl ExpressionNodeVariableFactory for str {
    fn as_var_expr(&self, row: u32, col: u32) -> ExpressionNode {
        Expression::VariableName(Name::from(self)).at_rc(row, col)
    }
}
