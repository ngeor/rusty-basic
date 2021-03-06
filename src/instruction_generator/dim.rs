use super::{Instruction, InstructionGenerator, RootPath};
use crate::common::*;
use crate::parser::{
    ArrayDimension, DimList, DimName, DimNameNode, DimType, ExpressionType, HasExpressionType,
    Name, TypeQualifier,
};

impl InstructionGenerator {
    pub fn generate_dim_instructions(&mut self, dim_list: DimList) {
        let DimList { shared, variables } = dim_list;
        for dim_name_node in variables {
            self.generate_dim_name_instructions(dim_name_node, shared);
        }
    }

    fn generate_dim_name_instructions(&mut self, dim_name_node: DimNameNode, shared: bool) {
        let Locatable {
            element: dim_name,
            pos,
        } = dim_name_node;

        // check if it is already defined to prevent re-allocation of STATIC variables
        let is_in_static_subprogram = self.is_in_static_subprogram();
        if is_in_static_subprogram {
            self.push(
                Instruction::IsVariableDefined(dim_name.clone(), shared),
                pos,
            );
            self.jump_if_false("begin-dim", pos);
            self.jump("end-dim", pos);
            self.label("begin-dim", pos);
            self.generate_dim_name(dim_name, shared, pos);
            self.label("end-dim", pos);
        } else {
            self.generate_dim_name(dim_name, shared, pos);
        }
    }

    fn is_in_static_subprogram(&self) -> bool {
        match &self.current_subprogram {
            Some(subprogram_name) => {
                self.subprogram_info_repository
                    .get_subprogram_info(subprogram_name)
                    .is_static
            }
            _ => false,
        }
    }

    fn generate_dim_name(&mut self, dim_name: DimName, shared: bool, pos: Location) {
        let DimName {
            bare_name,
            dim_type,
        } = dim_name;
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
    }
}
