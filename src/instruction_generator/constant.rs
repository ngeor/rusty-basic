use super::{Instruction, InstructionGenerator};
use crate::common::Locatable;
use crate::linter::{ExpressionNode, TypeDefinition};
use crate::parser::{HasQualifier, QualifiedNameNode};

impl InstructionGenerator {
    pub fn generate_const_instructions(&mut self, left: QualifiedNameNode, right: ExpressionNode) {
        let Locatable {
            element: qualified_name,
            pos,
        } = left;
        let left_type = qualified_name.qualifier();
        self.generate_expression_instructions_casting(right, TypeDefinition::BuiltIn(left_type));
        self.push(Instruction::StoreConst(qualified_name), pos);
    }
}
