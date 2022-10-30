use super::post_conversion_linter::PostConversionLinter;
use rusty_common::*;
use rusty_parser::{ExpressionType, HasExpressionType, PrintArg, PrintNode, TypeQualifier};

pub struct PrintLinter;

impl PostConversionLinter for PrintLinter {
    fn visit_print_node(&mut self, print_node: &PrintNode) -> Result<(), QErrorNode> {
        if let Some(f) = &print_node.format_string {
            if f.as_ref().expression_type() != ExpressionType::BuiltIn(TypeQualifier::DollarString)
            {
                return Err(QError::TypeMismatch).with_err_at(f);
            }
        }
        for print_arg in &print_node.args {
            if let PrintArg::Expression(expr_node) = print_arg {
                let type_definition = expr_node.as_ref().expression_type();
                if let ExpressionType::UserDefined(_) = type_definition {
                    return Err(QError::TypeMismatch).with_err_at(expr_node);
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
        assert_linter_err!(input, QError::TypeMismatch, 8, 15);
    }
}
