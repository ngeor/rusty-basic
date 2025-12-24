use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::specific::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    // must have 5 arguments:
    // filename
    // file mode
    // file access
    // file number
    // rec len
    if args.len() != 5 {
        return Err(LintError::ArgumentCountMismatch.at_no_pos());
    }
    args.require_string_argument(0)?;
    for i in 1..args.len() {
        args.require_integer_argument(i)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_linter_err;

    #[test]
    fn test_open_filename_must_be_string() {
        let program = "OPEN 42 AS #1";
        assert_linter_err!(program, LintError::ArgumentTypeMismatch, 1, 6);
    }

    #[test]
    fn test_rec_len_must_be_numeric() {
        let program = r#"OPEN "a.txt" AS #1 LEN = "hi""#;
        assert_linter_err!(program, LintError::ArgumentTypeMismatch, 1, 26);
    }
}
