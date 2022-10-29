use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    // must have 5 arguments:
    // filename
    // file mode
    // file access
    // file number
    // rec len
    if args.len() != 5 {
        return Err(QError::ArgumentCountMismatch).with_err_no_pos();
    }
    args.require_string_argument(0)?;
    for i in 1..args.len() {
        args.require_integer_argument(i)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::*;

    #[test]
    fn test_open_filename_must_be_string() {
        let program = "OPEN 42 AS #1";
        assert_linter_err!(program, QError::ArgumentTypeMismatch, 1, 6);
    }

    #[test]
    fn test_rec_len_must_be_numeric() {
        let program = r#"OPEN "a.txt" AS #1 LEN = "hi""#;
        assert_linter_err!(program, QError::ArgumentTypeMismatch, 1, 26);
    }
}
