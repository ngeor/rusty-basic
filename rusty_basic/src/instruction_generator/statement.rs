use super::{AddressOrLabel, Instruction, InstructionGenerator, Visitor};
use rusty_common::*;
use rusty_parser::{OnErrorOption, ResumeOption, Statement, StatementPos, Statements};

impl Visitor<Statements> for InstructionGenerator {
    fn visit(&mut self, block: Statements) {
        for s in block {
            self.visit(s);
        }
    }
}

impl Visitor<StatementPos> for InstructionGenerator {
    fn visit(&mut self, statement_pos: StatementPos) {
        let Positioned {
            element: statement,
            pos,
        } = statement_pos;

        if let Statement::Comment(_) = &statement {
        } else {
            self.mark_statement_address();
        }

        match statement {
            Statement::Assignment(a) => self.generate_assignment_instructions(a, pos),
            Statement::Const(_) => {
                // The CONST statement does not generate any instructions,
                // because the linter has replaced expressions that reference constants
                // with their actual value.
            }
            Statement::SubCall(sub_call) => self.generate_sub_call_instructions(sub_call, pos),
            Statement::BuiltInSubCall(sub_call) => {
                self.generate_built_in_sub_call_instructions(sub_call, pos)
            }
            Statement::Print(print) => {
                self.generate_print_instructions(print, pos);
            }
            Statement::IfBlock(i) => self.generate_if_block_instructions(i, pos),
            Statement::SelectCase(s) => self.generate_select_case_instructions(s, pos),
            Statement::ForLoop(f) => self.generate_for_loop_instructions(f, pos),
            Statement::While(w) => self.generate_while_instructions(w, pos),
            Statement::DoLoop(do_loop) => self.generate_do_loop_instructions(do_loop, pos),
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
                    Instruction::Return(opt_label.map(AddressOrLabel::Unresolved)),
                    pos,
                );
            }
            Statement::Exit(_) => {
                self.push(Instruction::PopRet, pos);
            }
            Statement::Comment(_) => {}
            Statement::Dim(dim_list) => {
                self.visit_dim_list(dim_list);
            }
            Statement::Redim(dim_list) => {
                self.visit_redim_list(dim_list);
            }
            Statement::End | Statement::System => {
                self.push(Instruction::Halt, pos);
            }
        }
    }
}
