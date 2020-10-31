use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::linter::{
    ArrayDimension, DimName, DimNameNode, DimType, ExpressionType, HasExpressionType,
};
use crate::parser::{Name, TypeQualifier};

impl InstructionGenerator {
    pub fn generate_dim_instructions(&mut self, dim_name_node: DimNameNode) {
        let Locatable {
            element: DimName {
                bare_name,
                dim_type,
            },
            pos,
        } = dim_name_node;
        match dim_type {
            DimType::Array(array_dimensions, box_element_type) => {
                self.push(Instruction::BeginCollectArguments, pos);

                for ArrayDimension { lbound, ubound } in array_dimensions {
                    let lbound_pos = lbound.pos();
                    self.generate_expression_instructions(lbound);
                    self.push(Instruction::PushUnnamed, lbound_pos);

                    let ubound_pos = ubound.pos();
                    self.generate_expression_instructions(ubound);
                    self.push(Instruction::PushUnnamed, ubound_pos);
                }

                let element_type = box_element_type.expression_type();

                let opt_q = match &element_type {
                    ExpressionType::BuiltIn(q) => Some(*q),
                    ExpressionType::FixedLengthString(_) => Some(TypeQualifier::DollarString),
                    _ => None,
                };

                self.push(Instruction::AllocateArray(element_type), pos);

                self.push(Instruction::VarPathName(Name::new(bare_name, opt_q)), pos);
                self.push(Instruction::CopyAToVarPath, pos);
            }
            DimType::BuiltIn(q) => {
                self.push(Instruction::AllocateBuiltIn(q), pos);
                self.push(Instruction::VarPathName(Name::new(bare_name, Some(q))), pos);
                self.push(Instruction::CopyAToVarPath, pos);
            }
            DimType::FixedLengthString(len) => {
                self.push(Instruction::AllocateFixedLengthString(len), pos);
                self.push(
                    Instruction::VarPathName(Name::new(
                        bare_name,
                        Some(TypeQualifier::DollarString),
                    )),
                    pos,
                );
                self.push(Instruction::CopyAToVarPath, pos);
            }
            DimType::UserDefined(user_defined_type_name) => {
                self.push(
                    Instruction::AllocateUserDefined(user_defined_type_name),
                    pos,
                );
                self.push(Instruction::VarPathName(Name::new(bare_name, None)), pos);
                self.push(Instruction::CopyAToVarPath, pos);
            }
            _ => self.push(
                Instruction::Dim(DimName {
                    bare_name,
                    dim_type,
                }),
                pos,
            ),
        }
    }
}
