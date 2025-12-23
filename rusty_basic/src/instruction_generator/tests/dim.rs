use crate::instruction_generator::test_utils::*;
use crate::instruction_generator::{Instruction, RootPath};
use rusty_common::AtPos;
use rusty_parser::specific::TypeQualifier;

#[test]
fn test_declaration_built_in_bare() {
    let input = r#"
    DIM X
    "#;
    assert_eq!(
        generate_instructions_str(input),
        [
            Instruction::AllocateBuiltIn(TypeQualifier::BangSingle).at_rc(2, 9),
            Instruction::VarPathName(RootPath {
                name: "X!".into(),
                shared: false
            })
            .at_rc(2, 9),
            Instruction::CopyAToVarPath.at_rc(2, 9),
            Instruction::Halt.at_rc(u32::MAX, u32::MAX)
        ]
    );
}

#[test]
fn test_declaration_built_in_qualified() {
    let input = r#"
    DIM X%
    "#;
    assert_eq!(
        generate_instructions_str(input),
        [
            Instruction::AllocateBuiltIn(TypeQualifier::PercentInteger).at_rc(2, 9),
            Instruction::VarPathName(RootPath {
                name: "X%".into(),
                shared: false
            })
            .at_rc(2, 9),
            Instruction::CopyAToVarPath.at_rc(2, 9),
            Instruction::Halt.at_rc(u32::MAX, u32::MAX)
        ]
    );
}

#[test]
fn test_declaration_built_in_extended() {
    let input = r#"
    DIM X AS DOUBLE
    "#;
    assert_eq!(
        generate_instructions_str(input),
        [
            Instruction::AllocateBuiltIn(TypeQualifier::HashDouble).at_rc(2, 9),
            Instruction::VarPathName(RootPath {
                name: "X#".into(),
                shared: false
            })
            .at_rc(2, 9),
            Instruction::CopyAToVarPath.at_rc(2, 9),
            Instruction::Halt.at_rc(u32::MAX, u32::MAX)
        ]
    );
}

#[test]
fn test_declaration_built_in_fixed_length_string() {
    let input = r#"
    DIM X AS STRING * 5
    "#;
    assert_eq!(
        generate_instructions_str(input),
        [
            Instruction::AllocateFixedLengthString(5).at_rc(2, 9),
            Instruction::VarPathName(RootPath {
                name: "X$".into(),
                shared: false
            })
            .at_rc(2, 9),
            Instruction::CopyAToVarPath.at_rc(2, 9),
            Instruction::Halt.at_rc(u32::MAX, u32::MAX)
        ]
    );
}

#[test]
fn test_declaration_user_defined() {
    let input = r#"
    TYPE Card
        Suit AS STRING * 10
        Value AS INTEGER
    END TYPE
    DIM X AS Card
    "#;
    assert_eq!(
        generate_instructions_str(input),
        [
            Instruction::AllocateUserDefined("Card".into()).at_rc(6, 9),
            Instruction::VarPathName(RootPath {
                name: "X".into(),
                shared: false
            })
            .at_rc(6, 9),
            Instruction::CopyAToVarPath.at_rc(6, 9),
            Instruction::Halt.at_rc(u32::MAX, u32::MAX)
        ]
    );
}
