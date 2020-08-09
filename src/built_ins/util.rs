use crate::common::*;
use crate::linter::{ExpressionNode, TypeQualifier};

pub fn require_single_numeric_argument(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
    if args.len() != 1 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        let q = args[0].try_qualifier()?;
        if q == TypeQualifier::DollarString || q == TypeQualifier::FileHandle {
            Err(QError::ArgumentTypeMismatch).with_err_at(&args[0])
        } else {
            Ok(())
        }
    }
}

pub fn require_single_string_argument(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
    if args.len() != 1 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        require_string_argument(args, 0)
    }
}

pub fn require_string_argument(args: &Vec<ExpressionNode>, idx: usize) -> Result<(), QErrorNode> {
    let q = args[idx].try_qualifier()?;
    if q != TypeQualifier::DollarString {
        Err(QError::ArgumentTypeMismatch).with_err_at(&args[idx])
    } else {
        Ok(())
    }
}

pub fn require_integer_argument(args: &Vec<ExpressionNode>, idx: usize) -> Result<(), QErrorNode> {
    let q = args[idx].try_qualifier()?;
    if q != TypeQualifier::PercentInteger {
        Err(QError::ArgumentTypeMismatch).with_err_at(&args[idx])
    } else {
        Ok(())
    }
}
