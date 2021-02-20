use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn resume_missing_label() {
    let input = "
    RESUME Jump
    ";
    assert_linter_err!(input, QError::LabelNotDefined, 2, 5);
}
