use super::{Instruction, InstructionGenerator};
use crate::common::*;
use crate::parser::{Statement, StatementNode, StatementNodes};

impl InstructionGenerator {
    pub fn generate_block_instructions(&mut self, block: StatementNodes) {
        for s in block {
            self.generate_statement_node_instructions(s);
        }
    }

    pub fn generate_statement_node_instructions(&mut self, statement_node: StatementNode) {
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
            Statement::ErrorHandler(label) => {
                self.push(Instruction::SetUnresolvedErrorHandler(label), pos);
            }
            Statement::Label(name) => {
                self.push(Instruction::Label(name), pos);
            }
            Statement::GoTo(name) => {
                self.push(Instruction::UnresolvedJump(name), pos);
            }
            Statement::GoSub(label) => {
                self.push(Instruction::UnresolvedGoSub(label), pos);
            }
            Statement::Return(opt_label) => {
                self.push(Instruction::UnresolvedReturn(opt_label), pos);
            }
            Statement::Comment(_) => {}
            Statement::Dim(d) => {
                self.generate_dim_instructions(d);
            }
        }
    }
}
