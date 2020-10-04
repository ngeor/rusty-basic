use super::post_conversion_linter::PostConversionLinter;
use crate::common::{QError, QErrorNode, ToLocatableError};
use crate::linter::types::{ExpressionType, HasExpressionType};
use crate::linter::{PrintArg, PrintNode};

pub struct PrintLinter;

impl PostConversionLinter for PrintLinter {
    fn visit_print_node(&self, print_node: &PrintNode) -> Result<(), QErrorNode> {
        match &print_node.format_string {
            Some(f) => self.visit_expression(f)?,
            None => {}
        };
        for print_arg in &print_node.args {
            match print_arg {
                PrintArg::Expression(expr_node) => {
                    let type_definition = expr_node.expression_type();
                    match type_definition {
                        ExpressionType::UserDefined(_) => {
                            return Err(QError::TypeMismatch).with_err_at(expr_node);
                        }
                        _ => {}
                    }

                    self.visit_expression(expr_node)?;
                }
                _ => {}
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
