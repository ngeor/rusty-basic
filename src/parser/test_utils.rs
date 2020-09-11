use super::{parse_main_file, parse_main_str};
use crate::common::*;

use crate::parser::types::*;
use std::fs::File;

/// Parses the given string and demands success.
///
/// # Panics
///
/// If the parser has an error.
pub fn parse<T: AsRef<[u8]> + 'static>(input: T) -> ProgramNode {
    parse_main_str(input).expect("Could not parse program")
}

/// Parses the given file under the `fixtures` folder.
///
/// # Panics
///
/// If the files does not exist or if the parser has an error.
pub fn parse_file<S: AsRef<str>>(filename: S) -> ProgramNode {
    let file_path = format!("fixtures/{}", filename.as_ref());
    let f = File::open(file_path).expect("Could not read bas file");
    parse_main_file(f).expect("Could not parse program")
}

/// Parses the given string, expecting that it will fail.
/// Returns the error with location information.
///
/// # Panics
///
/// If the parser does not have an error.
pub fn parse_err_node<T: AsRef<[u8]> + 'static>(input: T) -> QErrorNode {
    parse_main_str(input).unwrap_err()
}

/// Parses the given string, expecting that it will fail.
/// Returns the error without location information.
///
/// # Panics
///
/// If the parser does not have an error.
pub fn parse_err<T: AsRef<[u8]> + 'static>(input: T) -> QError {
    parse_err_node(input).into_err()
}

pub trait DemandSingle<T> {
    fn demand_single(self) -> T;
}

impl<T> DemandSingle<T> for Vec<T> {
    fn demand_single(mut self) -> T {
        assert_eq!(1, self.len());
        self.pop().unwrap()
    }
}

pub trait DemandSingleStatement {
    fn demand_single_statement(self) -> Statement;
}

impl DemandSingleStatement for ProgramNode {
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

// ========================================================
// macros
// ========================================================

#[macro_export]
macro_rules! assert_sub_call {
    ($actual_statement: expr, $expected_name: expr) => {
        match $actual_statement {
            Statement::SubCall(actual_bare_name, actual_args) => {
                let expected_bare_name: crate::parser::types::BareName = $expected_name.into();
                assert_eq!(actual_bare_name, expected_bare_name);
                assert_eq!(actual_args.is_empty(), true);
            }
            _ => panic!("Expected sub call")
        }
    };

    ($actual_statement: expr, $expected_name: expr, $($arg: expr),+) => {
        match $actual_statement {
            Statement::SubCall(actual_bare_name, actual_args) => {
                let expected_bare_name: crate::parser::types::BareName = $expected_name.into();
                assert_eq!(actual_bare_name, expected_bare_name);
                let actual_args_no_loc: Vec<crate::parser::types::Expression> = actual_args.into_iter().map(|x| x.strip_location()).collect();
                assert_eq!(actual_args_no_loc, vec![$($arg),+]);
            }
            _ => panic!("Expected sub call")
        }
    };
}

#[macro_export]
macro_rules! assert_expression {
    ($left:expr, $right:expr) => {
        let program = parse(format!("PRINT {}", $left)).demand_single_statement();
        crate::assert_sub_call!(program, "PRINT", $right);
    };
}

#[macro_export]
macro_rules! assert_literal_expression {
    ($left:expr, $right:expr) => {
        crate::assert_expression!($left, crate::parser::types::Expression::from($right));
    };
}

#[macro_export]
macro_rules! assert_variable_expression {
    ($left:expr, $right:expr) => {
        crate::assert_expression!(
            $left,
            crate::parser::types::Expression::VariableName(crate::parser::types::Name::from(
                $right
            ))
        );
    };
}
