use rusty_common::{AtPos, Position, Positioned};
use rusty_parser::{Print, PrintArg};
use rusty_variant::V_FALSE;

use crate::instruction_generator::{Instruction, InstructionGenerator, PrinterType};

impl InstructionGenerator {
    pub fn generate_print_instructions(&mut self, print: Print, pos: Position) {
        self.generate_opt_file_handle_instructions(&print, pos);
        self.generate_opt_format_string_instructions(&print, pos);
        for print_arg in print.args {
            self.generate_print_arg_instructions(print_arg, pos);
        }
        self.push(Instruction::PrintEnd, pos);
    }

    fn generate_opt_file_handle_instructions(&mut self, print: &Print, pos: Position) {
        match print.file_number {
            Some(f) => {
                self.push(Instruction::PrintSetPrinterType(PrinterType::File), pos);
                self.push(Instruction::PrintSetFileHandle(f), pos);
            }
            None => {
                self.push(
                    Instruction::PrintSetPrinterType(if print.lpt1 {
                        PrinterType::LPrint
                    } else {
                        PrinterType::Print
                    }),
                    pos,
                );
            }
        }
    }

    fn generate_opt_format_string_instructions(&mut self, print: &Print, pos: Position) {
        match &print.format_string {
            Some(format_string) => {
                self.generate_expression_instructions(format_string.clone());
            }
            None => {
                self.push_load(V_FALSE, pos);
            }
        }
        self.push(Instruction::PrintSetFormatStringFromA, pos);
    }

    fn generate_print_arg_instructions(&mut self, print_arg: PrintArg, pos: Position) {
        match print_arg {
            PrintArg::Expression(Positioned { element: arg, pos }) => {
                self.generate_expression_instructions(arg.at_pos(pos));
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
