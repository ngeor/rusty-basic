use crate::built_ins::BuiltInFunction;
use crate::common::AtRowCol;
use crate::instruction_generator::test_utils::*;
use crate::instruction_generator::Instruction;
use crate::parser::{QualifiedName, TypeQualifier};
use std::convert::TryFrom;

#[test]
fn test_built_in_len_with_string_variable_parameter() {
    let input = r#"
    X = LEN(A$)
    "#;
    assert_eq!(
        generate_instructions_str(input),
        [
            // implicit dim X
            Instruction::AllocateBuiltIn(TypeQualifier::BangSingle).at_rc(2, 5),
            Instruction::VarPathName("X!".into()).at_rc(2, 5),
            Instruction::CopyAToVarPath.at_rc(2, 5),
            // implicit dim A$
            Instruction::AllocateBuiltIn(TypeQualifier::DollarString).at_rc(2, 13),
            Instruction::VarPathName("A$".into()).at_rc(2, 13),
            Instruction::CopyAToVarPath.at_rc(2, 13),
            // function call
            Instruction::BeginCollectArguments.at_rc(2, 9),
            Instruction::VarPathName("A$".into()).at_rc(2, 13),
            Instruction::CopyVarPathToA.at_rc(2, 13),
            Instruction::PushUnnamed.at_rc(2, 13),
            Instruction::PushStack.at_rc(2, 9),
            Instruction::BuiltInFunction(BuiltInFunction::Len).at_rc(2, 9),
            // after function call
            Instruction::EnqueueToReturnStack(0).at_rc(2, 13),
            Instruction::StashFunctionReturnValue(QualifiedName::try_from("LEN%").unwrap())
                .at_rc(2, 9),
            Instruction::PopStack.at_rc(2, 9),
            // assign to by-ref variables
            Instruction::DequeueFromReturnStack.at_rc(2, 13),
            Instruction::VarPathName("A$".into()).at_rc(2, 13),
            Instruction::CopyAToVarPath.at_rc(2, 13),
            // cast result
            Instruction::UnStashFunctionReturnValue.at_rc(2, 9),
            Instruction::Cast(TypeQualifier::BangSingle).at_rc(2, 9),
            // assignment
            Instruction::VarPathName("X!".into()).at_rc(2, 5),
            Instruction::CopyAToVarPath.at_rc(2, 5),
            Instruction::Halt.at_rc(std::u32::MAX, std::u32::MAX)
        ]
    );
}
