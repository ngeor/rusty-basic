use crate::common::{AtRowCol};
use crate::instruction_generator::test_utils::*;
use crate::instruction_generator::Instruction;
use crate::linter::{DimName, ExpressionType};
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
            Instruction::Load(Variant::VInteger(1)).at_rc(2, 11),
            Instruction::PushUnnamed.at_rc(2, 11),
            // ubound of first dimension
            Instruction::Load(Variant::VInteger(3)).at_rc(2, 16),
            Instruction::PushUnnamed.at_rc(2, 16),
            // lbound of second dimension
            Instruction::Load(Variant::VInteger(1)).at_rc(2, 19),
            Instruction::PushUnnamed.at_rc(2, 19),
            // ubound of second dimension
            Instruction::Load(Variant::VInteger(4)).at_rc(2, 24),
            Instruction::PushUnnamed.at_rc(2, 24),
            // allocate array into A
            Instruction::AllocateArray(ExpressionType::BuiltIn(TypeQualifier::BangSingle)).at_rc(2, 9),
            // store allocated array value into variable
            Instruction::Store(DimName::parse("X!")).at_rc(2, 9),
            Instruction::CopyAToPointer.at_rc(2, 5),
            Instruction::Halt.at_rc(std::u32::MAX, std::u32::MAX)
        ]
    );
}
