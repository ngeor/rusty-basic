use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::linter::{ArrayDimension, DimNameNode, DimType};

impl InstructionGenerator {
    pub fn generate_dim_instructions(&mut self, dim_name_node: DimNameNode) {
        let Locatable {
            element: dim_name,
            pos,
        } = dim_name_node;
        match dim_name.dim_type() {
            DimType::Array(array_dimensions, box_element_type) => {
                self.push(Instruction::BeginCollectArguments, pos);

                for ArrayDimension { lbound, ubound } in array_dimensions {
                    self.generate_expression_instructions(lbound.clone().at(pos));
                    self.push(Instruction::PushUnnamed, pos);
                    self.generate_expression_instructions(ubound.clone().at(pos));
                    self.push(Instruction::PushUnnamed, pos);
                }
                let element_type = box_element_type.as_ref().clone();
                self.push(Instruction::AllocateArray(element_type), pos);
                self.push(Instruction::Store(dim_name), pos);
                self.push(Instruction::CopyAToPointer, pos);
            }
            _ => self.push(Instruction::Dim(dim_name), pos),
        }
    }
}
