use crate::built_ins::BuiltInSub;
use crate::common::StripLocation;
use crate::instruction_generator::test_utils::*;
use crate::instruction_generator::Instruction;
use crate::variant::Variant;

#[test]
fn test_constant_definition_and_usage_in_print() {
    let program = r#"
    CONST X = "hello"
    PRINT X
    "#;
    assert_eq!(
        generate_instructions_str(program).strip_location(),
        [
            // print
            Instruction::BeginCollectArguments,
            // no file handle
            Instruction::LoadIntoA(Variant::VInteger(0)),
            Instruction::PushAToUnnamedArg,
            // no format string
            Instruction::LoadIntoA(Variant::VInteger(0)),
            Instruction::PushAToUnnamedArg,
            // PrintArg::Expression
            Instruction::LoadIntoA(Variant::VInteger(0)),
            Instruction::PushAToUnnamedArg,
            // the arg itself
            Instruction::LoadIntoA(Variant::VString("hello".to_owned())),
            Instruction::PushAToUnnamedArg,
            // push and go
            Instruction::PushStack,
            Instruction::BuiltInSub(BuiltInSub::Print),
            Instruction::PopStack,
            Instruction::Halt
        ]
    );
}
