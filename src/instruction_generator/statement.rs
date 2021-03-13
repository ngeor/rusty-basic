use super::{AddressOrLabel, Instruction, InstructionGenerator, InternalBuiltInSub, Visitor};
use crate::common::*;
use crate::parser::{
    FieldItem, FieldNode, GetPutNode, OnErrorOption, ResumeOption, Statement, StatementNode,
    StatementNodes,
};

impl Visitor<StatementNodes> for InstructionGenerator {
    fn visit(&mut self, block: StatementNodes) {
        for s in block {
            self.visit(s);
        }
    }
}

impl Visitor<StatementNode> for InstructionGenerator {
    fn visit(&mut self, statement_node: StatementNode) {
        let Locatable {
            element: statement,
            pos,
        } = statement_node;

        if let Statement::Comment(_) = &statement {
        } else {
            self.mark_statement_address();
        }

        match statement {
            Statement::Assignment(left_side, right_side) => {
                self.generate_assignment_instructions(left_side, right_side, pos)
            }
            Statement::Const(_, _) => panic!("Constants should have been reduced by const_reducer"),
            Statement::SubCall(n, args) => self.generate_sub_call_instructions(n.at(pos), args),
            Statement::BuiltInSubCall(n, args) => {
                self.generate_built_in_sub_call_instructions(n, args, pos)
            }
            Statement::Print(print_node) => {
                self.generate_print_instructions(print_node, pos);
            }
            Statement::Field(FieldNode {
                file_number:
                    Locatable {
                        element: file_number,
                        ..
                    },
                fields,
            }) => {
                self.push(Instruction::BeginCollectArguments, pos);
                self.push_load_unnamed_arg(file_number, pos);
                for FieldItem { width, name } in fields {
                    // calculate width into A
                    self.generate_expression_instructions(width);
                    self.push(Instruction::PushAToUnnamedArg, pos);
                    // push bare name into A
                    self.push_load_unnamed_arg(String::from(name.bare_name().clone()), name.pos);
                }
                self.push(Instruction::PushStack, pos);
                self.push(
                    Instruction::InternalBuiltInSub(InternalBuiltInSub::Field),
                    pos,
                );
                self.push(Instruction::PopStack, pos);
            }
            Statement::Get(GetPutNode {
                file_number:
                    Locatable {
                        element: file_number,
                        ..
                    },
                record_number,
                ..
            }) => {
                self.push(Instruction::BeginCollectArguments, pos);
                self.push_load_unnamed_arg(file_number, pos);
                self.generate_expression_instructions(record_number.unwrap());
                self.push(Instruction::PushAToUnnamedArg, pos);
                self.push(Instruction::PushStack, pos);
                self.push(
                    Instruction::InternalBuiltInSub(InternalBuiltInSub::Get),
                    pos,
                );
                self.push(Instruction::PopStack, pos);
            }
            Statement::Put(GetPutNode {
                file_number:
                    Locatable {
                        element: file_number,
                        ..
                    },
                record_number,
                ..
            }) => {
                self.push(Instruction::BeginCollectArguments, pos);
                self.push_load_unnamed_arg(file_number, pos);
                self.generate_expression_instructions(record_number.unwrap());
                self.push(Instruction::PushAToUnnamedArg, pos);
                self.push(Instruction::PushStack, pos);
                self.push(
                    Instruction::InternalBuiltInSub(InternalBuiltInSub::Put),
                    pos,
                );
                self.push(Instruction::PopStack, pos);
            }
            Statement::IfBlock(i) => self.generate_if_block_instructions(i, pos),
            Statement::SelectCase(s) => self.generate_select_case_instructions(s, pos),
            Statement::ForLoop(f) => self.generate_for_loop_instructions(f, pos),
            Statement::While(w) => self.generate_while_instructions(w, pos),
            Statement::DoLoop(do_loop_node) => {
                self.generate_do_loop_instructions(do_loop_node, pos)
            }
            Statement::OnError(on_error_option) => match on_error_option {
                OnErrorOption::Label(label) => {
                    self.push(
                        Instruction::OnErrorGoTo(AddressOrLabel::Unresolved(label)),
                        pos,
                    );
                }
                OnErrorOption::Next => {
                    self.push(Instruction::OnErrorResumeNext, pos);
                }
                OnErrorOption::Zero => {
                    self.push(Instruction::OnErrorGoToZero, pos);
                }
            },
            Statement::Label(name) => {
                self.push(Instruction::Label(name), pos);
            }
            Statement::GoTo(name) => {
                self.push(Instruction::Jump(AddressOrLabel::Unresolved(name)), pos);
            }
            Statement::GoSub(label) => {
                self.push(Instruction::GoSub(AddressOrLabel::Unresolved(label)), pos);
            }
            Statement::Resume(resume_option) => match resume_option {
                ResumeOption::Bare => {
                    self.push(Instruction::Resume, pos);
                }
                ResumeOption::Next => {
                    self.push(Instruction::ResumeNext, pos);
                }
                ResumeOption::Label(label) => {
                    self.push(
                        Instruction::ResumeLabel(AddressOrLabel::Unresolved(label)),
                        pos,
                    );
                }
            },
            Statement::Return(opt_label) => {
                self.push(
                    Instruction::Return(opt_label.map(|label| AddressOrLabel::Unresolved(label))),
                    pos,
                );
            }
            Statement::Exit(_) => {
                self.push(Instruction::PopRet, pos);
            }
            Statement::Comment(_) => {}
            Statement::Dim(dim_list) => {
                self.visit(dim_list);
            }
            Statement::End | Statement::System => {
                self.push(Instruction::Halt, pos);
            }
        }
    }
}
