use crate::common::*;
use crate::linter::{ExpressionNode, LinterError, LinterErrorNode, TypeQualifier};

pub fn require_single_numeric_argument(args: &Vec<ExpressionNode>) -> Result<(), LinterErrorNode> {
    if args.len() != 1 {
        Err(LinterError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        let q = args[0].try_qualifier()?;
        if q == TypeQualifier::DollarString || q == TypeQualifier::FileHandle {
            Err(LinterError::ArgumentTypeMismatch).with_err_at(&args[0])
        } else {
            Ok(())
        }
    }
}

pub fn require_single_string_argument(args: &Vec<ExpressionNode>) -> Result<(), LinterErrorNode> {
    if args.len() != 1 {
        Err(LinterError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        require_string_argument(args, 0)
    }
}

pub fn require_string_argument(
    args: &Vec<ExpressionNode>,
    idx: usize,
) -> Result<(), LinterErrorNode> {
    let q = args[idx].try_qualifier()?;
    if q != TypeQualifier::DollarString {
        Err(LinterError::ArgumentTypeMismatch).with_err_at(&args[idx])
    } else {
        Ok(())
    }
}

pub fn require_integer_argument(
    args: &Vec<ExpressionNode>,
    idx: usize,
) -> Result<(), LinterErrorNode> {
    let q = args[idx].try_qualifier()?;
    if q != TypeQualifier::PercentInteger {
        Err(LinterError::ArgumentTypeMismatch).with_err_at(&args[idx])
    } else {
        Ok(())
    }
}
