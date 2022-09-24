use crate::common::QErrorNode;
use crate::linter::lint;
use crate::parser::{ProgramNode, UserDefinedTypes};
use crate::parser::string_helper::parse_main_str;

/// Lints the given string and returns the results.
///
/// # Panics
///
/// Panics if the parser or the linter have an error.
pub fn linter_ok_with_types<T>(input: T) -> (ProgramNode, UserDefinedTypes)
where
    T: AsRef<[u8]> + 'static,
{
    let program = parse_main_str(input).unwrap();
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
    let program = parse_main_str(input).expect("Parser should succeed");
    lint(program).expect_err(format!("Linter should fail {}", msg).as_str())
}

#[macro_export]
macro_rules! assert_linter_err {
    ($program:expr, $expected_err:expr) => {
        match crate::linter::test_utils::linter_err($program, "") {
            crate::common::QErrorNode::Pos(actual_err, _) => {
                assert_eq!(actual_err, $expected_err);
            }
            _ => panic!("Should have an error location"),
        }
    };

    ($program:expr, $expected_err:expr, $msg:expr) => {
        match crate::linter::test_utils::linter_err($program, format!("{}", $msg).as_ref()) {
            crate::common::QErrorNode::Pos(actual_err, _) => {
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
        match crate::linter::test_utils::linter_err($program, "") {
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

#[macro_export]
macro_rules! assert_linter_ok_top_level_statements {
    ($program:expr, $($statement: expr),+) => {
        let top_level_token_nodes: Vec<crate::parser::TopLevelTokenNode> = crate::linter::test_utils::linter_ok($program);
        let top_level_statements: Vec<crate::parser::Statement> = top_level_token_nodes.into_iter()
            .map(|crate::common::Locatable { element, .. }| match element {
                crate::parser::TopLevelToken::Statement(s) => s,
                _ => {panic!("Expected only top level statements, found {:?}", element);}
            } )
            .collect();
        assert_eq!(top_level_statements, vec![$($statement),+]);
    };
}
