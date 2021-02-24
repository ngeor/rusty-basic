use crate::common::{AtLocation, Locatable, Location};
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::parser::{PrintArg, PrintNode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrintHandle {
    Print,
    LPrint,
    File,
}

impl InstructionGenerator {
    pub fn generate_print_instructions(&mut self, print_node: PrintNode, pos: Location) {
        self.generate_opt_file_handle_instructions(&print_node, pos);
        self.generate_opt_format_string_instructions(&print_node, pos);
        for print_arg in print_node.args {
            self.generate_print_arg_instructions(print_arg, pos);
        }
        self.push(Instruction::PrintEnd, pos);
    }

    fn generate_opt_file_handle_instructions(&mut self, print_node: &PrintNode, pos: Location) {
        match print_node.file_number {
            Some(f) => {
                self.push(Instruction::PrintSetPrintHandle(PrintHandle::File), pos);
                self.push(Instruction::PrintSetFileHandle(f), pos);
            }
            None => {
                self.push(
                    Instruction::PrintSetPrintHandle(if print_node.lpt1 {
                        PrintHandle::LPrint
                    } else {
                        PrintHandle::Print
                    }),
                    pos,
                );
            }
        }
    }

    fn generate_opt_format_string_instructions(&mut self, print_node: &PrintNode, pos: Location) {
        match &print_node.format_string {
            Some(format_string) => {
                self.generate_expression_instructions(format_string.clone());
            }
            None => {
                self.push_load(false, pos);
            }
        }
        self.push(Instruction::PrintSetFormatStringFromA, pos);
    }

    fn generate_print_arg_instructions(&mut self, print_arg: PrintArg, pos: Location) {
        match print_arg {
            PrintArg::Expression(Locatable { element: arg, pos }) => {
                self.generate_expression_instructions(arg.at(pos));
                self.push(Instruction::PrintValueFromA, pos);
            }
            PrintArg::Comma => {
                self.push(Instruction::PrintComma, pos);
            }
            PrintArg::Semicolon => {
                self.push(Instruction::PrintSemicolon, pos);
            }
        }
    }
}
