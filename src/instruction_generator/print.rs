use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, FileHandle, Locatable, Location};
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::linter::{PrintArg, PrintNode};

pub const FLAG_EXPRESSION: u8 = 0;
pub const FLAG_COMMA: u8 = 1;
pub const FLAG_SEMICOLON: u8 = 2;

impl InstructionGenerator {
    pub fn generate_print_instructions(&mut self, print_node: PrintNode, pos: Location) {
        self.push(Instruction::BeginCollectArguments, pos);
        self.generate_opt_file_handle_instructions(print_node.file_number, pos);

        for print_arg in print_node.args {
            self.generate_print_arg_instructions(print_arg, pos);
        }

        self.push(Instruction::PushStack, pos);
        self.push(Instruction::BuiltInSub(BuiltInSub::Print), pos);
        self.push(Instruction::PopStack(None), pos);
    }

    pub fn generate_opt_file_handle_instructions(
        &mut self,
        opt_file_handle: Option<FileHandle>,
        pos: Location,
    ) {
        match opt_file_handle {
            Some(f) => {
                // first push true to indicate it has file handle
                self.push_load(true, pos);
                self.push(Instruction::PushUnnamed, pos);
                // then push the file handle itself
                self.push_load(f, pos);
                self.push(Instruction::PushUnnamed, pos);
            }
            None => {
                self.push_load(false, pos);
                self.push(Instruction::PushUnnamed, pos);
            }
        }
    }

    fn generate_print_arg_instructions(&mut self, print_arg: PrintArg, pos: Location) {
        match print_arg {
            PrintArg::Expression(Locatable { element: arg, pos }) => {
                self.push_load(FLAG_EXPRESSION, pos);
                self.push(Instruction::PushUnnamed, pos);
                self.generate_expression_instructions(arg.at(pos));
                self.push(Instruction::PushUnnamed, pos);
            }
            PrintArg::Comma => {
                self.push_load(FLAG_COMMA, pos);
                self.push(Instruction::PushUnnamed, pos);
            }
            PrintArg::Semicolon => {
                self.push_load(FLAG_SEMICOLON, pos);
                self.push(Instruction::PushUnnamed, pos);
            }
        }
    }
}
