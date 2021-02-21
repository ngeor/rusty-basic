use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::instruction_generator::AddressOrLabel;
use crate::parser::{OnErrorOption, ResumeOption, Statement, StatementNode, StatementNodes};

impl InstructionGenerator {
    pub fn mark_statement_address(&mut self) {
        self.statement_addresses.push(self.instructions.len());
    }

    pub fn generate_block_instructions(&mut self, block: StatementNodes) {
        for s in block {
            self.generate_statement_node_instructions(s);
        }
    }

    pub fn generate_statement_node_instructions(&mut self, statement_node: StatementNode) {
        // TODO fine tune for comments and other cases
        self.mark_statement_address();
        let Locatable {
            element: statement,
            pos,
        } = statement_node;
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
            Statement::IfBlock(i) => self.generate_if_block_instructions(i, pos),
            Statement::SelectCase(s) => self.generate_select_case_instructions(s, pos),
            Statement::ForLoop(f) => self.generate_for_loop_instructions(f, pos),
            Statement::While(w) => self.generate_while_instructions(w, pos),
            Statement::OnErrorGoTo(on_error_option) => match on_error_option {
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
            Statement::Dim(d) => {
                self.generate_dim_instructions(d);
            }
        }
    }
}
