use super::{Interpreter, Result, Stdlib};
use crate::parser::ConditionalBlockNode;

impl<S: Stdlib> Interpreter<S> {
    pub fn while_wend(&mut self, while_wend_block: &ConditionalBlockNode) -> Result<()> {
        while self.evaluate_condition(while_wend_block)? {
            self.statements(&while_wend_block.statements)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

    #[test]
    fn test_while_wend() {
        let input = "
        A = 1
        WHILE A < 5
            PRINT A
            A = A + 1
        WEND
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["1", "2", "3", "4"]);
    }
}
