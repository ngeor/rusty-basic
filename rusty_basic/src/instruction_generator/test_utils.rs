use crate::instruction_generator::{
    generate_instructions, Instruction, InstructionGeneratorResult, InstructionNode,
};
use rusty_common::Locatable;
use rusty_linter::{lint, HasUserDefinedTypes};
use rusty_parser::test_utils::parse;

pub fn generate_instructions_str_with_types<T>(
    input: T,
) -> (InstructionGeneratorResult, impl HasUserDefinedTypes)
where
    T: AsRef<[u8]> + 'static,
{
    let program = parse(input);
    let (linted_program, user_defined_types_holder) = lint(program).expect("Linter should succeed");
    (
        generate_instructions(linted_program),
        user_defined_types_holder,
    )
}

pub fn generate_instructions_str<T>(input: T) -> Vec<InstructionNode>
where
    T: AsRef<[u8]> + 'static,
{
    let (instruction_nodes, _) = generate_instructions_str_with_types(input);
    instruction_nodes.instructions
}

pub fn generate_instructions_str_no_location<T>(input: T) -> Vec<Instruction>
where
    T: AsRef<[u8]> + 'static,
{
    Locatable::strip_location(generate_instructions_str(input))
}
