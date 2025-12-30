use std::fs::File;

use rusty_common::*;

use crate::error::{ParseError, ParseErrorPos};
use crate::specific::*;
use crate::{parse, parse_main_file, parse_main_str};

pub fn parse_str_no_pos(input: &str) -> Vec<GlobalStatement> {
    parse(input).no_pos()
}

/// Parses the given file under the `fixtures` folder.
///
/// # Panics
///
/// If the files does not exist or if the parser has an error.
pub fn parse_file(filename: &str) -> Program {
    let file_path = format!("../fixtures/{}", filename);
    let f = File::open(file_path).expect("Could not read bas file");
    parse_main_file(f).expect("Could not parse program")
}

pub fn parse_file_no_pos(filename: &str) -> Vec<GlobalStatement> {
    parse_file(filename).no_pos()
}

/// Parses the given string, expecting that it will fail.
/// Returns the error with position information.
///
/// # Panics
///
/// If the parser does not have an error.
pub fn parse_err_pos(input: &str) -> ParseErrorPos {
    parse_main_str(input.to_owned()).expect_err("Parser should have failed")
}

/// Parses the given string, expecting that it will fail.
/// Returns the error without position information.
///
/// # Panics
///
/// If the parser does not have an error.
pub fn parse_err(input: &str) -> ParseError {
    parse_err_pos(input).element()
}

pub trait DemandSingle<T> {
    fn demand_single(self) -> T;
}

impl<T> DemandSingle<T> for Vec<T> {
    fn demand_single(mut self) -> T {
        assert_eq!(
            self.len(),
            1,
            "Expected single element, found {}",
            self.len()
        );
        self.pop().unwrap()
    }
}

pub trait DemandSingleStatement {
    fn demand_single_statement(self) -> Statement;
}

impl DemandSingleStatement for Program {
    fn demand_single_statement(self) -> Statement {
        let t = self.demand_single();
        match t {
            Positioned {
                element: GlobalStatement::Statement(s),
                ..
            } => s,
            _ => panic!("Expected: statement, found {:?}", t),
        }
    }
}

//
// Create NamePos out of literals
//

pub trait NameFactory {
    fn as_name(&self, row: u32, col: u32) -> NamePos;
    fn as_bare_name(&self, row: u32, col: u32) -> BareNamePos;
}

impl NameFactory for str {
    fn as_name(&self, row: u32, col: u32) -> NamePos {
        Name::from(self).at_rc(row, col)
    }

    fn as_bare_name(&self, row: u32, col: u32) -> BareNamePos {
        BareNamePos::new(
            CaseInsensitiveString::new(self.to_string()),
            Position::new(row, col),
        )
    }
}

//
// Create ExpressionPos out of literals
//

pub trait ExpressionLiteralFactory {
    /// Creates an expression holding a literal.
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionPos;
}

impl ExpressionLiteralFactory for &str {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionPos {
        Expression::StringLiteral(self.to_string()).at_rc(row, col)
    }
}

impl ExpressionLiteralFactory for u8 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionPos {
        Expression::IntegerLiteral(*self as i32).at_rc(row, col)
    }
}

impl ExpressionLiteralFactory for i32 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionPos {
        Expression::IntegerLiteral(*self).at_rc(row, col)
    }
}

impl ExpressionLiteralFactory for f32 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionPos {
        Expression::SingleLiteral(*self).at_rc(row, col)
    }
}

impl ExpressionLiteralFactory for f64 {
    fn as_lit_expr(&self, row: u32, col: u32) -> ExpressionPos {
        Expression::DoubleLiteral(*self).at_rc(row, col)
    }
}

//
// Create ExpressionPos out of literals
//

pub trait ExpressionVariableFactory {
    /// Creates an expression holding a variable name.
    fn as_var_expr(&self, row: u32, col: u32) -> ExpressionPos;
}

impl ExpressionVariableFactory for str {
    fn as_var_expr(&self, row: u32, col: u32) -> ExpressionPos {
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
            Statement::SubCall(sub_call) => {
                let (actual_bare_name, actual_args) = sub_call.into();
                let expected_bare_name: $crate::specific::BareName = $expected_name.into();
                assert_eq!(actual_bare_name, expected_bare_name, "SubCall name mismatch");
                assert!(actual_args.is_empty(), "Expected no args in SubCall");
            }
            _ => panic!("Expected SubCall")
        }
    };

    ($actual_statement: expr, $expected_name: expr, $($arg: expr),+) => {
        match $actual_statement {
            Statement::SubCall(sub_call) => {
                let (actual_bare_name, actual_args) = sub_call.into();
                let expected_bare_name: $crate::specific::BareName = $expected_name.into();
                assert_eq!(actual_bare_name, expected_bare_name, "SubCall name mismatch");
                let actual_args_no_pos: Vec<$crate::specific::Expression> = rusty_common::NoPosContainer::no_pos(actual_args);
                assert_eq!(actual_args_no_pos, vec![$($arg),+]);
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
            Statement::BuiltInSubCall(sub_call) => {
                let (actual_name, actual_args) = sub_call.into();
                assert_eq!(actual_name, $expected_name);
                assert!(actual_args.is_empty(), "Expected no args");
            }
            _ => panic!("Expected built-in sub call {:?}", $expected_name)
        }
    };

    ($input: expr, $expected_name: expr, $($arg: expr),+) => {
        let result = parse($input).demand_single_statement();
        match result {
            Statement::BuiltInSubCall(sub_call) => {
                let (actual_name, actual_args) = sub_call.into();
                assert_eq!(actual_name, $expected_name);
                let actual_args_no_pos: Vec<$crate::specific::Expression> = rusty_common::NoPosContainer::no_pos(actual_args);
                assert_eq!(actual_args_no_pos, vec![$($arg),+]);
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
        $crate::assert_expression!($left, $crate::specific::Expression::from($right));
    };
}

#[macro_export]
macro_rules! assert_parser_err {
    // TODO use this more for syntax errors
    ($input:expr, $expected_err:literal) => {
        $crate::assert_parser_err!(
            $input,
            $crate::error::ParseError::syntax_error($expected_err)
        );
    };

    ($input:expr, $expected_err:expr) => {
        assert_eq!($crate::test_utils::parse_err($input), $expected_err);
    };

    ($input:expr, $expected_err:expr, $row:expr, $col:expr) => {
        assert_eq!(
            $crate::test_utils::parse_err_pos($input),
            rusty_common::AtPos::at_rc($expected_err, $row, $col)
        );
    };
}

#[macro_export]
macro_rules! assert_global_assignment {
    ($input:expr, $name_expr:expr) => {
        match $crate::parse($input).demand_single_statement() {
            Statement::Assignment(a) => {
                let (left, _) = a.into();
                assert_eq!(left, $name_expr);
            }
            _ => panic!("Expected: assignment"),
        }
    };
    ($input:expr, $name:expr, $value:expr) => {
        match $crate::parse($input).demand_single_statement() {
            Statement::Assignment(a) => {
                let (left, right) = a.into();
                assert_eq!(left, Expression::var_unresolved($name));
                assert_eq!(right.element, Expression::IntegerLiteral($value));
            }
            _ => panic!("Expected: assignment"),
        }
    };
}

#[macro_export]
macro_rules! assert_function_declaration {
    ($input:expr, $expected_function_name:expr, $expected_params:expr) => {
        match $crate::parse($input).demand_single().element() {
            $crate::specific::GlobalStatement::FunctionDeclaration(
                $crate::specific::FunctionDeclaration { name, parameters },
            ) => {
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
                let parameters_no_pos: Vec<$crate::specific::Parameter> =
                    rusty_common::NoPosContainer::no_pos(parameters);
                for i in 0..parameters_no_pos.len() {
                    assert_eq!(parameters_no_pos[i], $expected_params[i], "Parameter {}", i);
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
            $crate::specific::GlobalStatement::DefType(def_type) => {
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
            $crate::specific::Statement::Dim($crate::specific::DimList {
                shared: false,
                variables: vec![$crate::specific::DimNameBuilder::new()
                    .bare_name($name)
                    .dim_type($crate::specific::DimType::BuiltIn(
                        TypeQualifier::$qualifier,
                        $crate::specific::BuiltInStyle::Extended
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
                $crate::specific::DimVar::new_compact_local($name, TypeQualifier::$qualifier)
                    .into_list_rc(1, 5)
            )
        );
    };
}

#[macro_export]
macro_rules! assert_file_handle {
    ($input:expr, $expected_file_handle:expr) => {
        let result: Statement = $crate::parse($input).demand_single_statement();
        match result {
            Statement::BuiltInSubCall(sub_call) => {
                let (_, args) = sub_call.into();
                assert_eq!(
                    args.into_iter().next().unwrap().element,
                    Expression::IntegerLiteral($expected_file_handle)
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
        rusty_common::Positioned::new(int_lit!($value), rusty_common::Position::new($row, $col))
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
        rusty_common::Positioned::new(bin_exp!($left ; plus $right), rusty_common::Position::new($row, $col))
    };
}

#[macro_export]
macro_rules! paren_exp {
    ($child: expr ; at $row: literal:$col: literal) => {
        rusty_common::Positioned::new(
            Expression::Parenthesis(Box::new($child)),
            rusty_common::Position::new($row, $col),
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
