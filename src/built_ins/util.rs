use crate::linter::{err_l, err_no_pos, Error, ExpressionNode, LinterError, TypeQualifier};

pub fn require_single_numeric_argument(args: &Vec<ExpressionNode>) -> Result<(), Error> {
    if args.len() != 1 {
        err_no_pos(LinterError::ArgumentCountMismatch)
    } else {
        let q = args[0].try_qualifier()?;
        if q == TypeQualifier::DollarString || q == TypeQualifier::FileHandle {
            err_l(LinterError::ArgumentTypeMismatch, &args[0])
        } else {
            Ok(())
        }
    }
}

pub fn require_single_string_argument(args: &Vec<ExpressionNode>) -> Result<(), Error> {
    if args.len() != 1 {
        err_no_pos(LinterError::ArgumentCountMismatch)
    } else {
        require_string_argument(args, 0)
    }
}

pub fn require_string_argument(args: &Vec<ExpressionNode>, idx: usize) -> Result<(), Error> {
    let q = args[idx].try_qualifier()?;
    if q != TypeQualifier::DollarString {
        err_l(LinterError::ArgumentTypeMismatch, &args[idx])
    } else {
        Ok(())
    }
}

pub fn require_integer_argument(args: &Vec<ExpressionNode>, idx: usize) -> Result<(), Error> {
    let q = args[idx].try_qualifier()?;
    if q != TypeQualifier::PercentInteger {
        err_l(LinterError::ArgumentTypeMismatch, &args[idx])
    } else {
        Ok(())
    }
}
