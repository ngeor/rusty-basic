use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, Locatable, Location};
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::linter::{PrintArg, PrintNode};

impl InstructionGenerator {
    pub fn generate_print_instructions(&mut self, print_node: PrintNode, pos: Location) {
        self.push(Instruction::BeginCollectArguments, pos);

        match print_node.file_number {
            Some(f) => {
                self.push(Instruction::Load(true.into()), pos);
                self.push(Instruction::PushUnnamed, pos);
                self.push(Instruction::Load(f.into()), pos);
                self.push(Instruction::PushUnnamed, pos);
            }
            None => {
                self.push(Instruction::Load(false.into()), pos);
                self.push(Instruction::PushUnnamed, pos);
            }
        }

        for print_arg in print_node.args {
            match print_arg {
                PrintArg::Expression(Locatable { element: arg, pos }) => {
                    self.generate_expression_instructions(arg.at(pos));
                    self.push(Instruction::PushUnnamed, pos);
                }
                _ => {}
            }
        }

        self.push(Instruction::PushStack, pos);
        self.push(Instruction::BuiltInSub(BuiltInSub::Print), pos);
        self.push(Instruction::PopStack(None), pos);
    }
}
