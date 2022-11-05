use crate::{lint, HasUserDefinedTypes};
use rusty_common::QErrorPos;
use rusty_parser::{parse, Program};

/// Lints the given string and returns the results.
///
/// # Panics
///
/// Panics if the parser or the linter have an error.
pub fn linter_ok_with_types(input: &str) -> (Program, impl HasUserDefinedTypes) {
    let program = parse(input);
    lint(program).unwrap()
}

/// Lints the given string and returns the linted program.
///
/// # Panics
///
/// Panics if the parser or the linter have an error.
pub fn linter_ok(input: &str) -> Program {
    linter_ok_with_types(input).0
}

/// Lints the given string and returns the error of the linter.
///
/// # Panics
///
/// If the parser has an error or if the linter did not have an error.
pub fn linter_err(input: &str, msg: &str) -> QErrorPos {
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
            QErrorPos::Pos(actual_err, _) => {
                assert_eq!(actual_err, $expected_err);
            }
            _ => panic!("Should have an error position"),
        }
    };

    ($program:expr, $expected_err:expr, $msg:expr) => {
        match $crate::test_utils::linter_err($program, format!("{}", $msg).as_ref()) {
            QErrorPos::Pos(actual_err, _) => {
                assert_eq!(
                    actual_err, $expected_err,
                    "'{}' failed, expected {:?} but was {:?}",
                    $msg, $expected_err, actual_err
                );
            }
            _ => panic!("Should have an error position"),
        }
    };

    ($program:expr, $expected_err:expr, $expected_row:expr, $expected_col:expr) => {
        match $crate::test_utils::linter_err($program, "") {
            QErrorPos::Pos(actual_err, actual_pos) => {
                assert_eq!(actual_err, $expected_err);
                assert_eq!(
                    actual_pos,
                    rusty_common::Position::new($expected_row, $expected_col)
                );
            }
            _ => panic!("Should have an error position"),
        }
    };
}

#[macro_export]
macro_rules! assert_linter_ok_global_statements {
    ($program:expr, $($statement: expr),+) => {
        let program: rusty_parser::Program = $crate::test_utils::linter_ok($program);
        let global_statements: Vec<rusty_parser::Statement> = program.into_iter()
            .map(|rusty_common::Positioned { element, .. }| match element {
                rusty_parser::GlobalStatement::Statement(s) => s,
                _ => {panic!("Expected only top level statements, found {:?}", element);}
            } )
            .collect();
        assert_eq!(global_statements, vec![$($statement),+]);
    };
}
