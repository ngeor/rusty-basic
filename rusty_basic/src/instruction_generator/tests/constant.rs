use crate::instruction_generator::test_utils::*;
use crate::instruction_generator::{Instruction, PrinterType};
use crate::variant::Variant;

#[test]
fn test_constant_definition_and_usage_in_print() {
    let program = r#"
    CONST X = "hello"
    PRINT X
    "#;
    assert_eq!(
        generate_instructions_str_no_location(program),
        [
            // print
            Instruction::PrintSetPrinterType(PrinterType::Print),
            // no format string
            Instruction::LoadIntoA(Variant::VInteger(0)),
            Instruction::PrintSetFormatStringFromA,
            // the arg
            Instruction::LoadIntoA(Variant::VString("hello".to_owned())),
            Instruction::PrintValueFromA,
            // end print
            Instruction::PrintEnd,
            Instruction::Halt
        ]
    );
}
