use crate::linter;
use crate::parser::parse_main_str;

pub fn linter_ok<T>(input: T) -> linter::ProgramNode
where
    T: AsRef<[u8]> + 'static,
{
    let program = parse_main_str(input).unwrap();
    linter::lint(program).unwrap()
}

pub fn linter_err<T>(input: T) -> crate::common::QErrorNode
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
