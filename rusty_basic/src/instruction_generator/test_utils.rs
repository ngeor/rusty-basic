use rusty_common::NoPosContainer;
use rusty_linter::core::lint;
use rusty_parser::{UserDefinedTypes, parse};

use crate::instruction_generator::{
    Instruction, InstructionGeneratorResult, InstructionPos, generate_instructions,
    unwrap_linter_context,
};

pub fn generate_instructions_str_with_types(
    input: &str,
) -> (InstructionGeneratorResult, UserDefinedTypes) {
    let program = parse(input);
    let (linted_program, linter_context) = lint(program).expect("Linter should succeed");
    let (linter_names, user_defined_types) = unwrap_linter_context(linter_context);
    (
        generate_instructions(linted_program, linter_names),
        user_defined_types,
    )
}

pub fn generate_instructions_str(input: &str) -> Vec<InstructionPos> {
    let (instruction_generator_result, _) = generate_instructions_str_with_types(input);
    instruction_generator_result.instructions
}

pub fn generate_instructions_str_no_pos(input: &str) -> Vec<Instruction> {
    generate_instructions_str(input).no_pos()
}
