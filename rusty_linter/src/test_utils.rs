use crate::{lint, HasUserDefinedTypes};
use rusty_common::QErrorNode;
use rusty_parser::test_utils::parse;
use rusty_parser::ProgramNode;

/// Lints the given string and returns the results.
///
/// # Panics
///
/// Panics if the parser or the linter have an error.
pub fn linter_ok_with_types<T>(input: T) -> (ProgramNode, impl HasUserDefinedTypes)
where
    T: AsRef<[u8]> + 'static,
{
    let program = parse(input);
    lint(program).unwrap()
}

/// Lints the given string and returns the linted program node.
///
/// # Panics
///
/// Panics if the parser or the linter have an error.
pub fn linter_ok<T>(input: T) -> ProgramNode
where
    T: AsRef<[u8]> + 'static,
{
    linter_ok_with_types(input).0
}

/// Lints the given string and returns the error of the linter.
///
/// # Panics
///
/// If the parser has an error or if the linter did not have an error.
pub fn linter_err<T>(input: T, msg: &str) -> QErrorNode
where
    T: AsRef<[u8]> + 'static,
{
    let program = parse(input);
    match lint(program) {
        Ok(_) => panic!("Linter should fail {}", msg),
        Err(e) => e,
    }
}

#[macro_export]
macro_rules! assert_linter_err {
    ($program:expr, $expected_err:expr) => {
        match $crate::test_utils::linter_err($program, "") {
            QErrorNode::Pos(actual_err, _) => {
                assert_eq!(actual_err, $expected_err);
            }
            _ => panic!("Should have an error location"),
        }
    };

    ($program:expr, $expected_err:expr, $msg:expr) => {
        match $crate::test_utils::linter_err($program, format!("{}", $msg).as_ref()) {
            QErrorNode::Pos(actual_err, _) => {
                assert_eq!(
                    actual_err, $expected_err,
                    "'{}' failed, expected {:?} but was {:?}",
                    $msg, $expected_err, actual_err
                );
            }
            _ => panic!("Should have an error location"),
        }
    };

    ($program:expr, $expected_err:expr, $expected_row:expr, $expected_col:expr) => {
        match $crate::test_utils::linter_err($program, "") {
            QErrorNode::Pos(actual_err, actual_pos) => {
                assert_eq!(actual_err, $expected_err);
                assert_eq!(actual_pos, Location::new($expected_row, $expected_col));
            }
            _ => panic!("Should have an error location"),
        }
    };
}

#[macro_export]
macro_rules! assert_linter_ok_top_level_statements {
    ($program:expr, $($statement: expr),+) => {
        let top_level_token_nodes: Vec<rusty_parser::TopLevelTokenNode> = $crate::test_utils::linter_ok($program);
        let top_level_statements: Vec<rusty_parser::Statement> = top_level_token_nodes.into_iter()
            .map(|Locatable { element, .. }| match element {
                rusty_parser::TopLevelToken::Statement(s) => s,
                _ => {panic!("Expected only top level statements, found {:?}", element);}
            } )
            .collect();
        assert_eq!(top_level_statements, vec![$($statement),+]);
    };
}
