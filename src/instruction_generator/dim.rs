use super::{Instruction, InstructionGenerator, RootPath};
use crate::common::*;
use crate::parser::{
    ArrayDimension, DimName, DimNameNode, DimType, ExpressionType, HasExpressionType, Name,
    TypeQualifier,
};

impl InstructionGenerator {
    pub fn generate_dim_instructions(&mut self, dim_name_node: DimNameNode) {
        // check if it is already defined to prevent re-allocation of STATIC variables
        let is_in_static_subprogram = match self.current_subprogram.as_ref() {
            Some(subprogram_name) => {
                self.subprogram_parameters
                    .get_subprogram_info(subprogram_name)
                    .is_static
            }
            _ => false,
        };
        if is_in_static_subprogram {
            self.push(
                Instruction::IsVariableDefined(dim_name_node.element.clone()),
                dim_name_node.pos(),
            );
        }
        let Locatable {
            element:
                DimName {
                    bare_name,
                    dim_type,
                    shared,
                },
            pos,
        } = dim_name_node;
        if is_in_static_subprogram {
            self.jump_if_false("begin-dim", pos);
            self.jump("end-dim", pos);
            self.label("begin-dim", pos);
        }
        match dim_type {
            DimType::Array(array_dimensions, box_element_type) => {
                self.push(Instruction::BeginCollectArguments, pos);

                for ArrayDimension { lbound, ubound } in array_dimensions {
                    if let Some(lbound) = lbound {
                        let lbound_pos = lbound.pos();
                        self.generate_expression_instructions(lbound);
                        self.push(Instruction::PushAToUnnamedArg, lbound_pos);
                    } else {
                        self.push_load_unnamed_arg(0, pos);
                    }

                    let ubound_pos = ubound.pos();
                    self.generate_expression_instructions(ubound);
                    self.push(Instruction::PushAToUnnamedArg, ubound_pos);
                }

                let element_type = box_element_type.expression_type();

                let opt_q = match &element_type {
                    ExpressionType::BuiltIn(q) => Some(*q),
                    ExpressionType::FixedLengthString(_) => Some(TypeQualifier::DollarString),
                    _ => None,
                };

                self.push(Instruction::AllocateArrayIntoA(element_type), pos);

                self.push(
                    Instruction::VarPathName(RootPath {
                        name: Name::new(bare_name, opt_q),
                        shared,
                    }),
                    pos,
                );
                self.push(Instruction::CopyAToVarPath, pos);
            }
            DimType::BuiltIn(q, _) => {
                self.push(Instruction::AllocateBuiltIn(q), pos);
                self.push(
                    Instruction::VarPathName(RootPath {
                        name: Name::new(bare_name, Some(q)),
                        shared,
                    }),
                    pos,
                );
                self.push(Instruction::CopyAToVarPath, pos);
            }
            DimType::FixedLengthString(_, len) => {
                self.push(Instruction::AllocateFixedLengthString(len), pos);
                self.push(
                    Instruction::VarPathName(RootPath {
                        name: Name::new(bare_name, Some(TypeQualifier::DollarString)),
                        shared,
                    }),
                    pos,
                );
                self.push(Instruction::CopyAToVarPath, pos);
            }
            DimType::UserDefined(Locatable {
                element: user_defined_type_name,
                ..
            }) => {
                self.push(
                    Instruction::AllocateUserDefined(user_defined_type_name),
                    pos,
                );
                self.push(
                    Instruction::VarPathName(RootPath {
                        name: Name::new(bare_name, None),
                        shared,
                    }),
                    pos,
                );
                self.push(Instruction::CopyAToVarPath, pos);
            }
            DimType::Bare => panic!("Unresolved type"),
        }
        if is_in_static_subprogram {
            self.label("end-dim", pos);
        }
    }
}
