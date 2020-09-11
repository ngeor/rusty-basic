use crate::common::{CaseInsensitiveString, QErrorNode};
use crate::linter;
use crate::linter::ResolvedUserDefinedType;
use crate::parser::parse_main_str;
use std::collections::HashMap;

/// Lints the given string and returns the results.
///
/// # Panics
///
/// Panics if the parser or the linter have an error.
pub fn linter_ok_with_types<T>(
    input: T,
) -> (
    linter::ProgramNode,
    HashMap<CaseInsensitiveString, ResolvedUserDefinedType>,
)
where
    T: AsRef<[u8]> + 'static,
{
    let program = parse_main_str(input).unwrap();
    linter::lint(program).unwrap()
}

/// Lints the given string and returns the linted program node.
///
/// # Panics
///
/// Panics if the parser or the linter have an error.
pub fn linter_ok<T>(input: T) -> linter::ProgramNode
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
pub fn linter_err<T>(input: T) -> QErrorNode
where
    T: AsRef<[u8]> + 'static,
{
    let program = parse_main_str(input).unwrap();
    linter::lint(program).unwrap_err()
}

#[macro_export]
macro_rules! assert_linter_err {
    ($program:expr, $expected_err:expr) => {
        match crate::linter::test_utils::linter_err($program) {
            crate::common::QErrorNode::Pos(actual_err, _) => {
                assert_eq!(actual_err, $expected_err);
            }
            _ => panic!("Should have an error location"),
        }
    };

    ($program:expr, $expected_err:expr, $expected_row:expr, $expected_col:expr) => {
        match crate::linter::test_utils::linter_err($program) {
            crate::common::QErrorNode::Pos(actual_err, actual_pos) => {
                assert_eq!(actual_err, $expected_err);
                assert_eq!(
                    actual_pos,
                    crate::common::Location::new($expected_row, $expected_col)
                );
            }
            _ => panic!("Should have an error location"),
        }
    };
}
