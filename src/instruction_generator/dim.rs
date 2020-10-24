use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::linter::{DimName, DimNameNode, DimType};
use crate::parser::{QualifiedName, TypeQualifier};

impl InstructionGenerator {
    pub fn generate_dim_instructions(&mut self, dim_name_node: DimNameNode) {
        let Locatable { element, pos } = dim_name_node;
        let (bare_name, dim_type) = element.into_inner();
        match dim_type {
            DimType::Array(array_dimensions, box_element_type) => {
                self.push(Instruction::BeginCollectArguments, pos);

                for array_dimension in array_dimensions {
                    self.generate_expression_instructions(array_dimension.lbound.at(pos));
                    self.push(Instruction::PushUnnamed, pos);
                    self.generate_expression_instructions(array_dimension.ubound.at(pos));
                    self.push(Instruction::PushUnnamed, pos);
                }

                self.push(Instruction::PushStack, pos);
                self.push(Instruction::AllocateArray(*box_element_type), pos);
                self.push(
                    Instruction::PopStack(Some(QualifiedName::new(
                        "_AllocateArray".into(),
                        TypeQualifier::PercentInteger,
                    ))),
                    pos,
                );
            }
            _ => {
                let element = DimName::new(bare_name, dim_type);
                self.push(Instruction::Dim(element), pos)
            }
        }
    }
}
