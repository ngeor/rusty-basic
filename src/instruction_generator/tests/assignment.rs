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
            // implicit dim
            Instruction::AllocateBuiltIn(TypeQualifier::BangSingle).at_rc(1, 1),
            Instruction::VarPathName("X!".into()).at_rc(1, 1),
            Instruction::CopyAToVarPath.at_rc(1, 1),
            // assignment with casting
            Instruction::LoadIntoA(Variant::VInteger(1)).at_rc(1, 5),
            Instruction::Cast(TypeQualifier::BangSingle).at_rc(1, 5),
            Instruction::VarPathName("X!".into()).at_rc(1, 1),
            Instruction::CopyAToVarPath.at_rc(1, 1),
            Instruction::Halt.at_rc(std::u32::MAX, std::u32::MAX)
        ]
    );
}

#[test]
fn test_assignment_no_cast_implicit_variable() {
    assert_eq!(
        generate_instructions_str("X% = 1").strip_location(),
        [
            // implicit dim
            Instruction::AllocateBuiltIn(TypeQualifier::PercentInteger),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            // assign
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            Instruction::Halt
        ]
    );
}

#[test]
fn test_assignment_no_cast_explicit_variable() {
    let input = r#"
    DIM X%
    X% = 1
    "#;
    assert_eq!(
        generate_instructions_str(input).strip_location(),
        [
            // dim
            Instruction::AllocateBuiltIn(TypeQualifier::PercentInteger),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            // assign
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            Instruction::Halt
        ]
    );
}

#[test]
fn test_assignment_no_cast_implicit_variable_implicit_dim_is_only_once() {
    let input = r#"
    X% = 1
    X% = 2
    "#;
    assert_eq!(
        generate_instructions_str(input).strip_location(),
        [
            // implicit dim
            Instruction::AllocateBuiltIn(TypeQualifier::PercentInteger),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            // assign
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            // assign
            Instruction::LoadIntoA(Variant::VInteger(2)),
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
            // implicit dim
            Instruction::AllocateBuiltIn(TypeQualifier::PercentInteger),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            // evaluation of binary expression
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::PushAToValueStack,
            Instruction::LoadIntoA(Variant::VSingle(2.1)),
            Instruction::CopyAToB,
            Instruction::PopValueStackIntoA,
            Instruction::Plus,
            // assignment with casting
            Instruction::Cast(TypeQualifier::PercentInteger),
            Instruction::VarPathName("X%".into()),
            Instruction::CopyAToVarPath,
            Instruction::Halt
        ]
    );
}
