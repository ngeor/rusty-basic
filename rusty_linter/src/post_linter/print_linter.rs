use super::post_conversion_linter::PostConversionLinter;
use crate::error::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::specific::{ExpressionType, HasExpressionType, Print, PrintArg, TypeQualifier};

pub struct PrintLinter;

impl PostConversionLinter for PrintLinter {
    fn visit_print(&mut self, print: &Print) -> Result<(), LintErrorPos> {
        if let Some(f) = &print.format_string {
            if f.expression_type() != ExpressionType::BuiltIn(TypeQualifier::DollarString) {
                return Err(LintError::TypeMismatch.at(f));
            }
        }
        for print_arg in &print.args {
            if let PrintArg::Expression(expr_pos) = print_arg {
                let type_definition = expr_pos.expression_type();
                if let ExpressionType::UserDefined(_) = type_definition {
                    return Err(LintError::TypeMismatch.at(expr_pos));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_linter_err;

    #[test]
    fn cannot_print_user_defined_type() {
        let input = "
        TYPE Card
            Suit AS STRING * 9
            Value AS INTEGER
        END TYPE

        DIM c AS Card
        PRINT c";
        assert_linter_err!(input, LintError::TypeMismatch, 8, 15);
    }
}
