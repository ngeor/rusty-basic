use crate::common::{AtRowCol, StripLocation};
use crate::instruction_generator::test_utils::*;
use crate::instruction_generator::Instruction;
use crate::parser::TypeQualifier;
use crate::variant::Variant;

#[test]
fn test_assignment() {
    assert_eq!(
        generate_instructions_str("X = 1"),
        [
            Instruction::Load(Variant::VInteger(1)).at_rc(1, 5),
            Instruction::Cast(TypeQualifier::BangSingle).at_rc(1, 5),
            Instruction::VarPathName("X!".into()).at_rc(1, 1),
            Instruction::CopyAToVarPath.at_rc(1, 1),
            Instruction::Halt.at_rc(std::u32::MAX, std::u32::MAX)
        ]
    );
}

#[test]
fn test_assignment_no_cast() {
    assert_eq!(
        generate_instructions_str("X% = 1").strip_location(),
        [
            Instruction::Load(Variant::VInteger(1)),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            Instruction::Halt
        ]
    );
}

#[test]
fn test_assignment_binary_plus() {
    assert_eq!(
        generate_instructions_str("X% = 1 + 2.1").strip_location(),
        [
            Instruction::PushRegisters,
            Instruction::Load(Variant::VInteger(1)),
            Instruction::CopyAToB,
            Instruction::Load(Variant::VSingle(2.1)),
            Instruction::SwapAWithB,
            Instruction::Plus,
            Instruction::PopRegisters,
            Instruction::Cast(TypeQualifier::PercentInteger),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            Instruction::Halt
        ]
    );
}
