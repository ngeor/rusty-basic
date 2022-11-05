use super::{Instruction, InstructionGenerator, RootPath};
use rusty_common::*;
use rusty_parser::*;
use rusty_variant::Variant;

impl InstructionGenerator {
    pub fn visit_dim_list(&mut self, item: DimList) {
        let DimList { shared, variables } = item;
        for dim_var_pos in variables {
            self.visit_dim_var_pos(dim_var_pos, false, shared);
        }
    }

    pub fn visit_redim_list(&mut self, item: DimList) {
        let DimList { shared, variables } = item;
        for dim_var_pos in variables {
            self.visit_dim_var_pos(dim_var_pos, true, shared);
        }
    }

    fn visit_dim_var_pos(&mut self, item: DimVarPos, is_redim: bool, shared: bool) {
        let Positioned {
            element: dim_name,
            pos,
        } = item;
        // check if it is already defined to prevent re-allocation of STATIC variables
        let is_in_static_subprogram = self.is_in_static_subprogram();
        if is_in_static_subprogram && !is_redim {
            debug_assert!(
                !shared,
                "Should not be possible to have a SHARED variable inside a function/sub"
            );
            self.push(Instruction::IsVariableDefined(dim_name.clone()), pos);
            self.jump_if_false("begin-dim", pos);
            self.jump("end-dim", pos);
            self.label("begin-dim", pos);
            self.generate_dim_name(dim_name, shared, pos);
            self.label("end-dim", pos);
        } else {
            self.generate_dim_name(dim_name, shared, pos);
        }
    }
}

impl InstructionGenerator {
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

    fn generate_dim_name(&mut self, dim_name: DimVar, shared: bool, pos: Position) {
        let DimVar {
            bare_name,
            var_type: dim_type,
        } = dim_name;
        match dim_type {
            DimType::Array(array_dimensions, box_element_type) => {
                self.push(Instruction::BeginCollectArguments, pos);

                for ArrayDimension { lbound, ubound } in array_dimensions {
                    if let Some(lbound) = lbound {
                        let lbound_pos = lbound.pos();
                        self.generate_expression_instructions(lbound);
                        self.push(Instruction::PushUnnamedByVal, lbound_pos);
                    } else {
                        self.push_load_unnamed_arg(Variant::VInteger(0), pos);
                    }

                    let ubound_pos = ubound.pos();
                    self.generate_expression_instructions(ubound);
                    self.push(Instruction::PushUnnamedByVal, ubound_pos);
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
                        name: Name::Qualified(bare_name, q),
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
                        name: Name::Qualified(bare_name, TypeQualifier::DollarString),
                        shared,
                    }),
                    pos,
                );
                self.push(Instruction::CopyAToVarPath, pos);
            }
            DimType::UserDefined(Positioned {
                element: user_defined_type_name,
                ..
            }) => {
                self.push(
                    Instruction::AllocateUserDefined(user_defined_type_name),
                    pos,
                );
                self.push(
                    Instruction::VarPathName(RootPath {
                        name: Name::Bare(bare_name),
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
