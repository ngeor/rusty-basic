use crate::common::{AtRowCol, StripLocation};
use crate::instruction_generator::test_utils::*;
use crate::instruction_generator::{AddressOrLabel, Instruction, PrinterType, RootPath};
use crate::parser::{BuiltInStyle, ExpressionType, ParamName, ParamType, TypeQualifier};
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
            Instruction::PushUnnamedByVal.at_rc(2, 11),
            // ubound of first dimension
            Instruction::LoadIntoA(Variant::VInteger(3)).at_rc(2, 16),
            Instruction::PushUnnamedByVal.at_rc(2, 16),
            // lbound of second dimension
            Instruction::LoadIntoA(Variant::VInteger(1)).at_rc(2, 19),
            Instruction::PushUnnamedByVal.at_rc(2, 19),
            // ubound of second dimension
            Instruction::LoadIntoA(Variant::VInteger(4)).at_rc(2, 24),
            Instruction::PushUnnamedByVal.at_rc(2, 24),
            // allocate array into A
            Instruction::AllocateArrayIntoA(ExpressionType::BuiltIn(TypeQualifier::BangSingle))
                .at_rc(2, 9),
            // store allocated array value into variable
            Instruction::VarPathName(RootPath {
                name: "X!".into(),
                shared: false
            })
            .at_rc(2, 9),
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
            Instruction::PushUnnamedByVal.at_rc(2, 12),
            // ubound of first dimension
            Instruction::LoadIntoA(Variant::VInteger(3)).at_rc(2, 17),
            Instruction::PushUnnamedByVal.at_rc(2, 17),
            // allocate array into A
            Instruction::AllocateArrayIntoA(ExpressionType::BuiltIn(TypeQualifier::PercentInteger))
                .at_rc(2, 9),
            // store allocated array value into variable
            Instruction::VarPathName(RootPath {
                name: "X%".into(),
                shared: false
            })
            .at_rc(2, 9),
            Instruction::CopyAToVarPath.at_rc(2, 9),
            // assignment to array
            // evaluate right side into A
            Instruction::LoadIntoA(Variant::VInteger(4)).at_rc(3, 13),
            // build name path
            Instruction::VarPathName(RootPath {
                name: "X%".into(),
                shared: false
            })
            .at_rc(3, 5),
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
            Instruction::PushUnnamedByVal,
            // ubound
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::PushUnnamedByVal,
            // allocate array into A
            Instruction::AllocateArrayIntoA(ExpressionType::BuiltIn(TypeQualifier::BangSingle)),
            Instruction::VarPathName(RootPath {
                name: "A!".into(),
                shared: false
            }),
            Instruction::CopyAToVarPath,
            // assign to array element
            // evaluate right side into A
            Instruction::LoadIntoA(Variant::VInteger(42)),
            Instruction::Cast(TypeQualifier::BangSingle),
            Instruction::VarPathName(RootPath {
                name: "A!".into(),
                shared: false
            }),
            Instruction::PushAToValueStack,
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::VarPathIndex,
            Instruction::PopValueStackIntoA,
            Instruction::CopyAToVarPath,
            // print it
            Instruction::PrintSetPrinterType(PrinterType::Print),
            Instruction::LoadIntoA(Variant::VInteger(0)),
            Instruction::PrintSetFormatStringFromA,
            Instruction::VarPathName(RootPath {
                name: "A!".into(),
                shared: false
            }),
            Instruction::PushAToValueStack,
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::VarPathIndex,
            Instruction::PopValueStackIntoA,
            Instruction::CopyVarPathToA,
            Instruction::PopVarPath,
            Instruction::PrintValueFromA,
            Instruction::PrintEnd,
            Instruction::Halt,
        ]
    );
}

#[test]
fn test_pass_param_to_sub() {
    let input = r#"
    DIM A(1 TO 1)
    Menu A()

    SUB Menu(values())
    END SUB
    "#;
    assert_eq!(
        generate_instructions_str(input).strip_location(),
        [
            // evaluate dimensions of array
            Instruction::BeginCollectArguments,
            // lbound
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::PushUnnamedByVal,
            // ubound
            Instruction::LoadIntoA(Variant::VInteger(1)),
            Instruction::PushUnnamedByVal,
            // allocate array into A
            Instruction::AllocateArrayIntoA(ExpressionType::BuiltIn(TypeQualifier::BangSingle)),
            Instruction::VarPathName(RootPath {
                name: "A!".into(),
                shared: false
            }),
            Instruction::CopyAToVarPath,
            // call sub
            Instruction::BeginCollectArguments,
            Instruction::VarPathName(RootPath {
                name: "A!".into(),
                shared: false
            }),
            Instruction::CopyVarPathToA,
            Instruction::PopVarPath,
            Instruction::PushNamed(ParamName::new(
                "values".into(),
                ParamType::Array(Box::new(ParamType::BuiltIn(
                    TypeQualifier::BangSingle,
                    BuiltInStyle::Compact
                )))
            )),
            Instruction::PushStack,
            Instruction::PushRet(16),
            Instruction::Jump(AddressOrLabel::Resolved(22)),
            Instruction::EnqueueToReturnStack(0),
            Instruction::PopStack,
            Instruction::DequeueFromReturnStack,
            Instruction::VarPathName(RootPath {
                name: "A!".into(),
                shared: false
            }),
            Instruction::CopyAToVarPath,
            Instruction::Halt,
            // sub implementation
            Instruction::Label(":sub:Menu".into()),
            Instruction::PopRet,
        ]
    );
}
