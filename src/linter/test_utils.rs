use crate::linter;
use crate::parser::parse_main_str;

pub fn linter_ok<T>(input: T) -> linter::ProgramNode
where
    T: AsRef<[u8]>,
{
    let program = parse_main_str(input).unwrap();
    linter::lint(program).unwrap()
}

pub fn linter_err<T>(input: T) -> linter::Error
where
    T: AsRef<[u8]>,
{
    let program = parse_main_str(input).unwrap();
    linter::lint(program).unwrap_err()
}

#[macro_export]
macro_rules! assert_linter_err {
    ($program:expr, $expected_msg:expr, $expected_row:expr, $expected_col:expr) => {
        let (actual_err, actual_pos) = crate::linter::test_utils::linter_err($program).consume();
        assert_eq!(actual_err, $expected_msg);
        assert_eq!(
            actual_pos.unwrap(),
            crate::common::Location::new($expected_row, $expected_col)
        );
    };
}
