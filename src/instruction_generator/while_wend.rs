use super::{InstructionGenerator, Visitor};
use crate::common::*;
use crate::parser::ConditionalBlockNode;

impl InstructionGenerator {
    pub fn generate_while_instructions(&mut self, w: ConditionalBlockNode, pos: Location) {
        self.label("while", pos);
        self.generate_expression_instructions(w.condition);
        self.jump_if_false("wend", pos);
        self.visit(w.statements);
        self.jump("while", pos);
        self.label("wend", pos);
    }
}
