use crate::instruction_generator::{
    generate_instructions, Instruction, InstructionGeneratorResult, InstructionNode,
};
use rusty_common::Locatable;
use rusty_linter::{lint, HasUserDefinedTypes};
use rusty_parser::parse;

pub fn generate_instructions_str_with_types(
    input: &str,
) -> (InstructionGeneratorResult, impl HasUserDefinedTypes) {
    let program = parse(input);
    let (linted_program, user_defined_types_holder) = lint(program).expect("Linter should succeed");
    (
        generate_instructions(linted_program),
        user_defined_types_holder,
    )
}

pub fn generate_instructions_str(input: &str) -> Vec<InstructionNode> {
    let (instruction_nodes, _) = generate_instructions_str_with_types(input);
    instruction_nodes.instructions
}

pub fn generate_instructions_str_no_location(input: &str) -> Vec<Instruction> {
    Locatable::strip_location(generate_instructions_str(input))
}
