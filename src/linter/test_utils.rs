#[macro_export]
macro_rules! assert_linter_err {
    ($program:expr, $expected_msg:expr, $expected_row:expr, $expected_col:expr) => {
        let (actual_err, actual_pos) =
            crate::interpreter::test_utils::linter_err($program).consume();
        assert_eq!(actual_err, $expected_msg);
        assert_eq!(
            actual_pos.unwrap(),
            crate::common::Location::new($expected_row, $expected_col)
        );
    };
}
