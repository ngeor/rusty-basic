use std::fs::File;

use crate::pc::{Parser, Tokenizer};
use crate::pc_specific::create_string_tokenizer;
use crate::types::*;
use crate::{parse, parse_main_file, parse_main_str};
use rusty_common::*;

pub fn parse_something<P>(input: &str, parser: impl Parser<Output = P>) -> Result<P, QError> {
    let mut tokenizer = create_string_tokenizer(input);
    parser.parse(&mut tokenizer)
}

pub fn parse_something_completely<P>(input: &str, parser: impl Parser<Output = P>) -> P {
    let mut tokenizer = create_string_tokenizer(input);
    let result = parser
        .parse(&mut tokenizer)
        .unwrap_or_else(|_| panic!("Should have succeeded for {}", input));
    assert!(
        tokenizer.read().expect("Should read EOF token").is_none(),
        "Should have parsed {} completely",
        input
    );
    result
}

pub fn parse_str_no_location(input: &str) -> Vec<TopLevelToken> {
    Locatable::strip_location(parse(input))
}

/// Parses the given file under the `fixtures` folder.
///
/// # Panics
///
/// If the files does not exist or if the parser has an error.
pub fn parse_file(filename: &str) -> ProgramNode {
    let file_path = format!("../fixtures/{}", filename);
    let f = File::open(file_path).expect("Could not read bas file");
    parse_main_file(f).expect("Could not parse program")
}

pub fn parse_file_no_location(filename: &str) -> Vec<TopLevelToken> {
    Locatable::strip_location(parse_file(filename))
}

/// Parses the given string, expecting that it will fail.
/// Returns the error with location information.
///
/// # Panics
///
/// If the parser does not have an error.
pub fn parse_err_node(input: &str) -> QErrorNode {
    parse_main_str(input).expect_err("Parser should have failed")
}

/// Parses the given string, expecting that it will fail.
/// Returns the error without location information.
///
/// # Panics
///
/// If the parser does not have an error.
pub fn parse_err(input: &str) -> QError {
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
            _ => panic!("Expected: statement, found {:?}", t),
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

impl ExpressionNodeLiteralFactory for u8 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionNode {
        Expression::IntegerLiteral(*self as i32).at_rc(row, col)
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
        Expression::var_unresolved(self).at_rc(row, col)
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
                let expected_bare_name: $crate::types::BareName = $expected_name.into();
                assert_eq!(actual_bare_name, expected_bare_name, "SubCall name mismatch");
                assert!(actual_args.is_empty(), "Expected no args in SubCall");
            }
            _ => panic!("Expected SubCall")
        }
    };

    ($actual_statement: expr, $expected_name: expr, $($arg: expr),+) => {
        match $actual_statement {
            Statement::SubCall(actual_bare_name, actual_args) => {
                let expected_bare_name: $crate::types::BareName = $expected_name.into();
                assert_eq!(actual_bare_name, expected_bare_name, "SubCall name mismatch");
                let actual_args_no_loc: Vec<$crate::Expression> = Locatable::strip_location(actual_args);
                assert_eq!(actual_args_no_loc, vec![$($arg),+]);
            }
            _ => panic!("Expected SubCall")
        }
    };
}

#[macro_export]
macro_rules! assert_built_in_sub_call {
    ($input: expr, $expected_name: expr) => {
        let result = parse($input).demand_single_statement();
        match result {
            Statement::BuiltInSubCall(actual_name, actual_args) => {
                assert_eq!(actual_name, $expected_name);
                assert!(actual_args.is_empty(), "Expected no args");
            }
            _ => panic!("Expected built-in sub call {:?}", $expected_name)
        }
    };

    ($input: expr, $expected_name: expr, $($arg: expr),+) => {
        let result = parse($input).demand_single_statement();
        match result {
            Statement::BuiltInSubCall(actual_name, actual_args) => {
                assert_eq!(actual_name, $expected_name);
                let actual_args_no_loc: Vec<$crate::Expression> = Locatable::strip_location(actual_args);
                assert_eq!(actual_args_no_loc, vec![$($arg),+]);
            }
            _ => panic!("Expected built-in sub call {:?}", $expected_name)
        }
    };
}

#[macro_export]
macro_rules! assert_expression {
    ($left:expr, $right:expr) => {
        let program = parse(&format!("Flint {}", $left)).demand_single_statement();
        $crate::assert_sub_call!(program, "Flint", $right);
    };

    (var $input:expr) => {
        $crate::assert_expression!($input, Expression::var_unresolved($input));
    };

    (fn $input:expr) => {
        $crate::assert_expression!(
            $input,
            Expression::FunctionCall(
                Name::from($input.split('(').next().unwrap()),
                vec![Expression::IntegerLiteral(1).at_rc(
                    1,
                    8 + $input.find('(').expect("Func expression should have paren") as u32
                )]
            )
        );
    };
}

#[macro_export]
macro_rules! assert_literal_expression {
    ($left:expr, $right:expr) => {
        $crate::assert_expression!($left, $crate::types::Expression::from($right));
    };
}

#[macro_export]
macro_rules! assert_parser_err {
    // TODO use this more for syntax errors
    ($input:expr, $expected_err:literal) => {
        $crate::assert_parser_err!($input, QError::syntax_error($expected_err));
    };

    ($input:expr, $expected_err:expr) => {
        assert_eq!($crate::test_utils::parse_err($input), $expected_err);
    };

    ($input:expr, $expected_err:expr, $row:expr, $col:expr) => {
        assert_eq!(
            $crate::test_utils::parse_err_node($input),
            QErrorNode::Pos($expected_err, Location::new($row, $col))
        );
    };
}

#[macro_export]
macro_rules! assert_top_level_assignment {
    ($input:expr, $name_expr:expr) => {
        match $crate::parse($input).demand_single_statement() {
            Statement::Assignment(n, _) => {
                assert_eq!(n, $name_expr);
            }
            _ => panic!("Expected: assignment"),
        }
    };
    ($input:expr, $name:expr, $value:expr) => {
        match $crate::parse($input).demand_single_statement() {
            Statement::Assignment(n, Locatable { element: v, .. }) => {
                assert_eq!(n, Expression::var_unresolved($name));
                assert_eq!(v, Expression::IntegerLiteral($value));
            }
            _ => panic!("Expected: assignment"),
        }
    };
}

#[macro_export]
macro_rules! assert_function_declaration {
    ($input:expr, $expected_function_name:expr, $expected_params:expr) => {
        match $crate::parse($input).demand_single().element() {
            TopLevelToken::FunctionDeclaration(name, parameters) => {
                assert_eq!(
                    name.element(),
                    $expected_function_name,
                    "Function name mismatch"
                );
                assert_eq!(
                    parameters.len(),
                    $expected_params.len(),
                    "Parameter count mismatch"
                );
                let parameters_without_location: Vec<ParamName> =
                    Locatable::strip_location(parameters);
                for i in 0..parameters_without_location.len() {
                    assert_eq!(
                        parameters_without_location[i], $expected_params[i],
                        "Parameter {}",
                        i
                    );
                }
            }
            _ => panic!("{:?}", $input),
        }
    };
}

// Asserts that the given input program contains a def type top level token.
#[macro_export]
macro_rules! assert_def_type {
    ($input:expr, $expected_qualifier:expr, $expected_ranges:expr) => {
        match $crate::parse($input).demand_single().element() {
            TopLevelToken::DefType(def_type) => {
                let def_type_qualifier = def_type.qualifier();
                assert_eq!(def_type_qualifier, $expected_qualifier);
                assert_eq!(def_type.ranges(), &$expected_ranges);
            }
            _ => panic!("{:?}", $input),
        }
    };
}

#[macro_export]
macro_rules! assert_parse_dim_extended_built_in {
    ($name: literal, $keyword: literal, $qualifier: ident) => {
        let input = format!("DIM {} AS {}", $name, $keyword);
        let p = $crate::parse(&input).demand_single_statement();
        assert_eq!(
            p,
            $crate::Statement::Dim($crate::DimList {
                shared: false,
                variables: vec![$crate::DimNameBuilder::new()
                    .bare_name($name)
                    .dim_type($crate::DimType::BuiltIn(
                        TypeQualifier::$qualifier,
                        $crate::BuiltInStyle::Extended
                    ))
                    .build()
                    .at_rc(1, 5)]
            })
        );
    };
}

#[macro_export]
macro_rules! assert_parse_dim_compact {
    ($name: literal) => {
        let input = format!("DIM {}", $name);
        let p = $crate::parse(&input).demand_single_statement();
        assert_eq!(
            p,
            Statement::Dim(
                DimNameBuilder::new()
                    .bare_name($name)
                    .dim_type(DimType::Bare)
                    .build_list_rc(1, 5)
            )
        );
    };

    ($name: literal, $keyword: literal, $qualifier: ident) => {
        let input = format!("DIM {}{}", $name, $keyword);
        let p = $crate::parse(&input).demand_single_statement();
        assert_eq!(
            p,
            Statement::Dim(
                DimName::new_compact_local($name, TypeQualifier::$qualifier).into_list_rc(1, 5)
            )
        );
    };
}

#[macro_export]
macro_rules! assert_file_handle {
    ($input:expr, $expected_file_handle:expr) => {
        let result: Statement = $crate::parse($input).demand_single_statement();
        match result {
            Statement::BuiltInSubCall(_, args) => {
                assert_eq!(
                    args[0].as_ref(),
                    &Expression::IntegerLiteral($expected_file_handle)
                );
            }
            _ => {
                panic!("Expected built-in sub call");
            }
        }
    };
}

// TODO use the new xxx_lit macros more

#[macro_export]
macro_rules! int_lit {
    ($value: literal) => {
        Expression::IntegerLiteral($value)
    };

    ($value: literal at $row: literal:$col: literal) => {
        Locatable::new(int_lit!($value), Location::new($row, $col))
    };
}

#[macro_export]
macro_rules! bin_exp {
    ($left: expr ; plus $right: expr) => {
        Expression::BinaryExpression(
            Operator::Plus,
            Box::new($left),
            Box::new($right),
            ExpressionType::Unresolved,
        )
    };

    ($left: expr ; plus $right: expr ; at $row: literal:$col: literal) => {
        Locatable::new(bin_exp!($left ; plus $right), Location::new($row, $col))
    };
}

#[macro_export]
macro_rules! paren_exp {
    ($child: expr ; at $row: literal:$col: literal) => {
        Locatable::new(
            Expression::Parenthesis(Box::new($child)),
            Location::new($row, $col),
        )
    };
}

#[macro_export]
macro_rules! expr {
    (var($name: literal)) => {
        Expression::Variable(Name::from($name), VariableInfo::unresolved())
    };

    (prop($first: literal.$second: literal)) => {
        Expression::Property(
            Box::new($crate::expr!(var($first))),
            Name::from($second),
            ExpressionType::Unresolved,
        )
    };

    (prop($first: literal, $second: literal, $third: literal)) => {
        Expression::Property(
            Box::new($crate::expr!(prop($first.$second))),
            Name::from($third),
            ExpressionType::Unresolved,
        )
    };

    (prop($first: expr, $second: literal)) => {
        Expression::Property(
            Box::new($first),
            Name::from($second),
            ExpressionType::Unresolved,
        )
    };

    (fn $name:expr, $arg:expr) => {
        Expression::FunctionCall(Name::from($name), vec![$arg])
    };
}

#[macro_export]
macro_rules! parametric_test {
    (
        $handler:ident,
        [
            $($test_name:ident, $value:literal),+$(,)?
        ]
    ) => {
        $(
            #[test]
            fn $test_name() {
                $handler($value);
            }
        )+
    };
}
