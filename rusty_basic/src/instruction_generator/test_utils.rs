use crate::instruction_generator::{
    generate_instructions, Instruction, InstructionGeneratorResult, InstructionPos,
};
use rusty_common::NoPosContainer;
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

pub fn generate_instructions_str(input: &str) -> Vec<InstructionPos> {
    let (instruction_generator_result, _) = generate_instructions_str_with_types(input);
    instruction_generator_result.instructions
}

pub fn generate_instructions_str_no_pos(input: &str) -> Vec<Instruction> {
    generate_instructions_str(input).no_pos()
}
