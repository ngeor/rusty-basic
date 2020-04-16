use super::{
    ExpressionNode, Name, NameNode, Parser, ParserError, ProgramNode, StatementNode,
    TopLevelTokenNode,
};
use crate::common::Location;
use std::fs::File;

/// Parses the given program and demands success.
pub fn parse(input: &str) -> ProgramNode {
    let mut parser = Parser::from(input);
    parser.parse().expect("Could not parse program")
}

pub fn parse_file<S: AsRef<str>>(filename: S) -> ProgramNode {
    let file_path = format!("fixtures/{}", filename.as_ref());
    let mut parser = Parser::from(File::open(file_path).expect("Could not read bas file"));
    parser.parse().expect("Could not parse program")
}

/// Parses the given input, expecting that it will fail.
/// Returns the lexer error.
/// Panics if parsing actually succeeded.
pub fn parse_err<T: AsRef<[u8]>>(input: T) -> ParserError {
    let mut parser = Parser::from(input);
    parser.parse().unwrap_err()
}

pub trait ProgramNodeHelper {
    /// Parses the given input and asserts that it is parsed successfully and that
    /// it contains a single top level token node.
    ///
    /// Return the single top level token node of the parsed program.
    fn demand_single(self) -> TopLevelTokenNode;

    fn demand_single_statement(self) -> StatementNode;
}

impl ProgramNodeHelper for ProgramNode {
    fn demand_single(mut self) -> TopLevelTokenNode {
        assert_eq!(1, self.len());
        self.pop().unwrap()
    }

    fn demand_single_statement(self) -> StatementNode {
        let t = self.demand_single();
        match t {
            TopLevelTokenNode::Statement(s) => s,
            _ => panic!(format!("Expected statement, found {:?}", t)),
        }
    }
}

//
// Create NameNode out of literals
//

pub trait NameNodeFactory {
    fn as_name(&self, row: u32, col: u32) -> NameNode;
}

impl NameNodeFactory for str {
    fn as_name(&self, row: u32, col: u32) -> NameNode {
        NameNode::new(Name::from(self), Location::new(row, col))
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
        ExpressionNode::StringLiteral(self.to_string(), Location::new(row, col))
    }
}

impl ExpressionNodeLiteralFactory for i32 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionNode {
        ExpressionNode::IntegerLiteral(*self, Location::new(row, col))
    }
}

impl ExpressionNodeLiteralFactory for f32 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionNode {
        ExpressionNode::SingleLiteral(*self, Location::new(row, col))
    }
}

impl ExpressionNodeLiteralFactory for f64 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionNode {
        ExpressionNode::DoubleLiteral(*self, Location::new(row, col))
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
        ExpressionNode::VariableName(self.as_name(row, col))
    }
}

//
// macros
//

/// Asserts the the left-side is an Expression Node that holds a Variable Name
/// of the given literal. The literal can be an untyped string like "A",
/// or a typed like "A$".
#[macro_export]
macro_rules! assert_var_expr {
    ($left: expr, $expected_var_name: literal) => {
        match &$left {
            ExpressionNode::VariableName(v) => assert_eq!(v, $expected_var_name),
            _ => panic!(format!("Expected variable name {}, found {:?}", $expected_var_name, $left))
        }
    };
}

/// Asserts a sub call statement.
#[macro_export]
macro_rules! assert_sub_call {
    // no args
    ($left: expr, $name: literal) => {
        match &$left {
            StatementNode::SubCall(n, args) => {
                assert_eq!(n, $name);
                assert_eq!(0, args.len());
            },
            _ => panic!(format!("Expected sub call {} without args, found {:?}", $name, $left))
        }
    };
    // one variable name arg
    ($left: expr, $name: literal, $arg: literal) => {
        match &$left {
            StatementNode::SubCall(n, args) => {
                assert_eq!(n, $name);
                assert_eq!(1, args.len());
                assert_var_expr!(args[0], $arg);
            },
            _ => panic!(format!("Expected sub call {} with one arg, found {:?}", $name, $left))
        }
    };
}

/// Asserts a top-level sub call statement.
#[macro_export]
macro_rules! assert_top_sub_call {
    ($left: expr, $name: literal) => {
        match &$left {
            TopLevelTokenNode::Statement(StatementNode::SubCall(n, args)) => {
                assert_eq!(n, $name);
                assert_eq!(0, args.len());
            },
            _ => panic!(format!("Expected top-level sub call {}, found {:?}", $name, $left))
        }
    };
}
