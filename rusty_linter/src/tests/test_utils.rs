use rusty_parser::{Program, parse};

use crate::core::LintErrorPos;
use crate::{LinterContext, lint};

/// Lints the given string and returns the results.
///
/// # Panics
///
/// Panics if the parser or the linter have an error.
pub fn linter_ok_with_types(input: &str) -> (Program, LinterContext) {
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
pub fn linter_err(input: &str, msg: &str) -> LintErrorPos {
    let program = parse(input);
    match lint(program) {
        Ok(_) => panic!("Linter should fail {}", msg),
        Err(e) => e,
    }
}

#[macro_export]
macro_rules! assert_linter_err {
    ($program:expr, $expected_err:expr) => {
        let rusty_common::Positioned { element, .. } =
            $crate::tests::test_utils::linter_err($program, "");
        assert_eq!(element, $expected_err);
    };

    ($program:expr, $expected_err:expr, $msg:expr) => {
        let rusty_common::Positioned { element, .. } =
            $crate::tests::test_utils::linter_err($program, format!("{}", $msg).as_ref());
        assert_eq!(
            element, $expected_err,
            "'{}' failed, expected {:?} but was {:?}",
            $msg, $expected_err, element
        );
    };

    ($program:expr, $expected_err:expr, $expected_row:expr, $expected_col:expr) => {
        let rusty_common::Positioned { element, pos } =
            $crate::tests::test_utils::linter_err($program, "");
        assert_eq!(element, $expected_err);
        assert_eq!(
            pos,
            rusty_common::Position::new($expected_row, $expected_col)
        );
    };
}

#[macro_export]
macro_rules! assert_linter_ok_global_statements {
    ($program:expr, $($statement: expr),+) => {
        let program: rusty_parser::Program = $crate::tests::test_utils::linter_ok($program);
        let global_statements: Vec<rusty_parser::Statement> = program.into_iter()
            .map(|rusty_common::Positioned { element, .. }| match element {
                rusty_parser::GlobalStatement::Statement(s) => s,
                _ => {panic!("Expected only top level statements, found {:?}", element);}
            } )
            .collect();
        assert_eq!(global_statements, vec![$($statement),+]);
    };
}
