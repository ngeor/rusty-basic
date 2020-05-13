use super::error::*;
use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::parser::TypeQualifier;

pub struct BuiltInSubLinter;

impl PostConversionLinter for BuiltInSubLinter {
    fn visit_built_in_sub_call(
        &self,
        n: &BuiltInSub,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), Error> {
        match n {
            BuiltInSub::System => {
                if args.len() != 0 {
                    err_no_pos(LinterError::ArgumentCountMismatch)
                } else {
                    Ok(())
                }
            }
            BuiltInSub::Environ => {
                if args.len() != 1 {
                    err_no_pos(LinterError::ArgumentCountMismatch)
                } else if args[0].as_ref().try_qualifier()? != TypeQualifier::DollarString {
                    err_l(LinterError::ArgumentTypeMismatch, &args[0])
                } else {
                    Ok(())
                }
            }
            BuiltInSub::Input => {
                if args.len() == 0 {
                    err_no_pos(LinterError::ArgumentCountMismatch)
                } else {
                    args.iter()
                        .map(|a| match a.as_ref() {
                            Expression::Variable(_) => Ok(()),
                            _ => err_l(LinterError::VariableRequired, a),
                        })
                        .collect()
                }
            }
            _ => Ok(()),
        }
    }
}
