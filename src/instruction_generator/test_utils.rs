use crate::instruction_generator::{
    generate_instructions, InstructionGeneratorResult, InstructionNode,
};
use crate::linter::lint;
use crate::parser::{parse_main_str, UserDefinedTypes};

pub fn generate_instructions_str_with_types<T>(
    input: T,
) -> (InstructionGeneratorResult, UserDefinedTypes)
where
    T: AsRef<[u8]> + 'static,
{
    let program = parse_main_str(input).expect("Parser should succeed");
    let (linted_program, user_defined_types) = lint(program).expect("Linter should succeed");
    (generate_instructions(linted_program), user_defined_types)
}

pub fn generate_instructions_str<T>(input: T) -> Vec<InstructionNode>
where
    T: AsRef<[u8]> + 'static,
{
    let (instruction_nodes, _) = generate_instructions_str_with_types(input);
    instruction_nodes.instructions
}
