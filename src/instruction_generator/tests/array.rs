use crate::built_ins::BuiltInSub;
use crate::common::{AtRowCol, StripLocation};
use crate::instruction_generator::test_utils::*;
use crate::instruction_generator::Instruction;
use crate::linter::ExpressionType;
use crate::parser::TypeQualifier;
use crate::variant::Variant;

#[test]
fn test_declaration() {
    let input = r#"
    DIM X(1 TO 3, 1 TO 4)
    "#;
    assert_eq!(
        generate_instructions_str(input),
        [
            // start collecting arguments (to evaluate the array dimensions)
            Instruction::BeginCollectArguments.at_rc(2, 9),
            // lbound of first dimension
            Instruction::LoadIntoA(Variant::VInteger(1)).at_rc(2, 11),
            Instruction::PushAToUnnamedArg.at_rc(2, 11),
            // ubound of first dimension
            Instruction::LoadIntoA(Variant::VInteger(3)).at_rc(2, 16),
            Instruction::PushAToUnnamedArg.at_rc(2, 16),
            // lbound of second dimension
            Instruction::LoadIntoA(Variant::VInteger(1)).at_rc(2, 19),
            Instruction::PushAToUnnamedArg.at_rc(2, 19),
            // ubound of second dimension
            Instruction::LoadIntoA(Variant::VInteger(4)).at_rc(2, 24),
            Instruction::PushAToUnnamedArg.at_rc(2, 24),
            // allocate array into A
            Instruction::AllocateArrayIntoA(ExpressionType::BuiltIn(TypeQualifier::BangSingle))
                .at_rc(2, 9),
            // store allocated array value into variable
            Instruction::VarPathName("X!".into()).at_rc(2, 9),
            Instruction::CopyAToVarPath.at_rc(2, 9),
            Instruction::Halt.at_rc(std::u32::MAX, std::u32::MAX)
        ]
    );
}

#[test]
fn test_assignment() {
    let input = r#"
    DIM X%(1 TO 3)
    X%(2) = 4
    "#;
    assert_eq!(
        generate_instructions_str(input),
        [
            // start collecting arguments (to evaluate the array dimensions)
            Instruction::BeginCollectArguments.at_rc(2, 9),
            // lbound of first dimension
            Instruction::LoadIntoA(Variant::VInteger(1)).at_rc(2, 12),
            Instruction::PushAToUnnamedArg.at_rc(2, 12),
            // ubound of first dimension
            Instruction::LoadIntoA(Variant::VInteger(3)).at_rc(2, 17),
            Instruction::PushAToUnnamedArg.at_rc(2, 17),
            // allocate array into A
            Instruction::AllocateArrayIntoA(ExpressionType::BuiltIn(TypeQualifier::PercentInteger))
                .at_rc(2, 9),
            // store allocated array value into variable
            Instruction::VarPathName("X%".into()).at_rc(2, 9),
            Instruction::CopyAToVarPath.at_rc(2, 9),
            // assignment to array
            // evaluate right side into A
            Instruction::LoadIntoA(Variant::VInteger(4)).at_rc(3, 13),
            // build name path
            Instruction::VarPathName("X%".into()).at_rc(3, 5),
            Instruction::PushAToValueStack.at_rc(3, 8),
            Instruction::LoadIntoA(Variant::VInteger(2)).at_rc(3, 8), // loads into A, therefore needs PushRegisters before
            Instruction::VarPathIndex.at_rc(3, 8),
            Instruction::PopValueStackIntoA.at_rc(3, 8),
            Instruction::CopyAToVarPath.at_rc(3, 5),
            Instruction::Halt.at_rc(std::u32::MAX, std::u32::MAX)
        ]
    );
}

#[test]
fn test_assign_and_print_one_element() {
    let input = r#"
    DIM A(1 TO 1)
    A(1) = 42
    PRINT A(1)
    "#;
    assert_eq!(
        generate_instructions_str(input).strip_location(),
        [
            // evaluate dimensions of array
            Instruction::BeginCollectArguments,
            // lbound
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::PushAToUnnamedArg,
            // ubound
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::PushAToUnnamedArg,
            // allocate array into A
            Instruction::AllocateArrayIntoA(ExpressionType::BuiltIn(TypeQualifier::BangSingle)),
            Instruction::VarPathName("A!".into()),
            Instruction::CopyAToVarPath,
            // assign to array element
            // evaluate right side into A
            Instruction::LoadIntoA(Variant::VInteger(42)),
            Instruction::Cast(TypeQualifier::BangSingle),
            Instruction::VarPathName("A!".into()),
            Instruction::PushAToValueStack,
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::VarPathIndex,
            Instruction::PopValueStackIntoA,
            Instruction::CopyAToVarPath,
            // print it
            Instruction::BeginCollectArguments,
            Instruction::LoadIntoA(Variant::VInteger(0)),
            Instruction::PushAToUnnamedArg,
            Instruction::LoadIntoA(Variant::VInteger(0)),
            Instruction::PushAToUnnamedArg,
            Instruction::LoadIntoA(Variant::VInteger(0)),
            Instruction::PushAToUnnamedArg,
            Instruction::VarPathName("A!".into()),
            Instruction::PushAToValueStack,
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::VarPathIndex,
            Instruction::PopValueStackIntoA,
            Instruction::CopyVarPathToA,
            Instruction::PushAToUnnamedArg,
            Instruction::PushStack,
            Instruction::BuiltInSub(BuiltInSub::Print),
            Instruction::PopStack,
            Instruction::Halt,
        ]
    );
}
