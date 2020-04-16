use super::*;
use crate::parser::IfBlockNode;
use std::convert::TryInto;

impl<S: Stdlib> Interpreter<S> {
    pub fn if_block(&mut self, if_block: &IfBlockNode) -> Result<()> {
        if self._conditional_block(&if_block.if_block)? {
            return Ok(());
        }

        for else_if_block in &if_block.else_if_blocks {
            if self._conditional_block(else_if_block)? {
                return Ok(());
            }
        }

        match &if_block.else_block {
            Some(e) => self.statements(&e),
            None => Ok(()),
        }
    }

    fn _conditional_block(&mut self, conditional_block: &ConditionalBlockNode) -> Result<bool> {
        let condition_expr: &ExpressionNode = &conditional_block.condition;
        let condition_value: Variant = self.evaluate_expression(condition_expr)?;
        let is_true: bool = condition_value
            .try_into()
            .map_err(|e| InterpreterError::new_with_pos(e, conditional_block.pos))?;
        if is_true {
            self.statements(&conditional_block.block)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

    #[test]
    fn test_if_block_true() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_block_false() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, Vec::<String>::new());
    }

    #[test]
    fn test_if_else_block_true() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSE
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_else_block_false() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSE
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["bye"]);
    }

    #[test]
    fn test_if_elseif_block_true_true() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_elseif_block_true_false() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSEIF 2 < 1 THEN
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_elseif_block_false_true() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["bye"]);
    }

    #[test]
    fn test_if_elseif_block_false_false() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 2 < 1 THEN
            PRINT \"bye\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, Vec::<String>::new());
    }

    #[test]
    fn test_if_elseif_else_block_true_true() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        ELSE
            PRINT \"else\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_elseif_else_block_true_false() {
        let input = "
        IF 1 < 2 THEN
            PRINT \"hello\"
        ELSEIF 2 < 1 THEN
            PRINT \"bye\"
        ELSE
            PRINT \"else\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
    }

    #[test]
    fn test_if_elseif_else_block_false_true() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        ELSE
            PRINT \"else\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["bye"]);
    }

    #[test]
    fn test_if_elseif_else_block_false_false() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 2 < 1 THEN
            PRINT \"bye\"
        ELSE
            PRINT \"else\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["else"]);
    }

    #[test]
    fn test_if_multiple_elseif_block() {
        let input = "
        IF 2 < 1 THEN
            PRINT \"hello\"
        ELSEIF 1 < 2 THEN
            PRINT \"bye\"
        ELSEIF 1 < 2 THEN
            PRINT \"else if 2\"
        END IF
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["bye"]);
    }

    #[test]
    fn test_single_line_if() {
        let input = r#"
        IF 1 THEN PRINT "hello"
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["hello"]);
        let input = r#"
        IF 0 THEN PRINT "hello"
        "#;
        assert_eq!(interpret(input).stdlib.output.len(), 0);
        let input = r#"
        PRINT "before"
        IF 1 THEN PRINT "hello"
        PRINT "after"
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["before", "hello", "after"]);
        let input = r#"
        PRINT "before"
        IF 0 THEN PRINT "hello"
        PRINT "after"
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["before", "after"]);
    }
}
