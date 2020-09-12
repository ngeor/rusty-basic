use super::{Instruction, InstructionGenerator};
use crate::common::Locatable;
use crate::linter::{ExpressionNode, HasQualifier, QualifiedNameNode, ResolvedTypeDefinition};

impl InstructionGenerator {
    pub fn generate_const_instructions(&mut self, left: QualifiedNameNode, right: ExpressionNode) {
        let Locatable {
            element: qualified_name,
            pos,
        } = left;

        let left_type = qualified_name.qualifier();
        let right_type = right.try_type_definition().unwrap();

        match right_type {
            ResolvedTypeDefinition::BuiltIn(q) => {
                if q != left_type {
                    self.push(Instruction::Cast(left_type), pos);
                }
            }
            _ => {
                panic!("Last part cannot be non-built-in type");
            }
        }

        self.generate_expression_instructions(right);
        self.push(Instruction::StoreConst(qualified_name), pos);
    }
}
