use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, Locatable, Location, QError};
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::linter::{PrintArg, PrintNode};
use crate::variant::Variant;
use std::convert::TryFrom;

pub enum PrintHandle {
    Print,
    LPrint,
    File,
}

impl From<PrintHandle> for u8 {
    fn from(print_handle: PrintHandle) -> Self {
        match print_handle {
            PrintHandle::Print => 0,
            PrintHandle::LPrint => 1,
            PrintHandle::File => 2,
        }
    }
}

impl From<PrintHandle> for Variant {
    fn from(print_handle: PrintHandle) -> Self {
        Self::from(u8::from(print_handle))
    }
}

impl TryFrom<u8> for PrintHandle {
    type Error = QError;
    fn try_from(encoded_print_handle: u8) -> Result<Self, Self::Error> {
        if encoded_print_handle == Self::Print.into() {
            Ok(Self::Print)
        } else if encoded_print_handle == Self::LPrint.into() {
            Ok(Self::LPrint)
        } else if encoded_print_handle == Self::File.into() {
            Ok(Self::File)
        } else {
            Err(QError::TypeMismatch)
        }
    }
}

impl TryFrom<&Variant> for PrintHandle {
    type Error = QError;
    fn try_from(encoded_print_handle: &Variant) -> Result<Self, Self::Error> {
        u8::try_from(encoded_print_handle).and_then(|x| PrintHandle::try_from(x))
    }
}

impl TryFrom<Variant> for PrintHandle {
    type Error = QError;
    fn try_from(encoded_print_handle: Variant) -> Result<Self, Self::Error> {
        u8::try_from(encoded_print_handle).and_then(|x| PrintHandle::try_from(x))
    }
}

pub enum PrintArgType {
    Expression,
    Comma,
    Semicolon,
}

impl From<PrintArgType> for u8 {
    fn from(print_arg_type: PrintArgType) -> Self {
        match print_arg_type {
            PrintArgType::Expression => 0,
            PrintArgType::Comma => 1,
            PrintArgType::Semicolon => 2,
        }
    }
}

impl From<PrintArgType> for Variant {
    fn from(print_arg_type: PrintArgType) -> Self {
        Self::from(u8::from(print_arg_type))
    }
}

impl TryFrom<u8> for PrintArgType {
    type Error = QError;
    fn try_from(encoded_print_arg_type: u8) -> Result<Self, Self::Error> {
        if encoded_print_arg_type == Self::Expression.into() {
            Ok(Self::Expression)
        } else if encoded_print_arg_type == Self::Comma.into() {
            Ok(Self::Comma)
        } else if encoded_print_arg_type == Self::Semicolon.into() {
            Ok(Self::Semicolon)
        } else {
            Err(QError::TypeMismatch)
        }
    }
}

impl TryFrom<&Variant> for PrintArgType {
    type Error = QError;
    fn try_from(encoded_print_arg_type: &Variant) -> Result<Self, Self::Error> {
        u8::try_from(encoded_print_arg_type).and_then(|x| PrintArgType::try_from(x))
    }
}

impl TryFrom<Variant> for PrintArgType {
    type Error = QError;
    fn try_from(encoded_print_arg_type: Variant) -> Result<Self, Self::Error> {
        u8::try_from(encoded_print_arg_type).and_then(|x| PrintArgType::try_from(x))
    }
}

impl InstructionGenerator {
    pub fn generate_print_instructions(&mut self, print_node: PrintNode, pos: Location) {
        self.push(Instruction::BeginCollectArguments, pos);
        self.generate_opt_file_handle_instructions(&print_node, pos);
        self.generate_opt_format_string_instructions(&print_node, pos);
        for print_arg in print_node.args {
            self.generate_print_arg_instructions(print_arg, pos);
        }

        self.push(Instruction::PushStack, pos);
        self.push(Instruction::BuiltInSub(BuiltInSub::Print), pos);
        self.push(Instruction::PopStack(None), pos);
    }

    fn generate_opt_file_handle_instructions(&mut self, print_node: &PrintNode, pos: Location) {
        match print_node.file_number {
            Some(f) => {
                // first push to indicate it has file handle
                self.push_load_unnamed_arg(PrintHandle::File, pos);
                // then push the file handle itself
                self.push_load_unnamed_arg(f, pos);
            }
            None => {
                self.push_load_unnamed_arg(
                    if print_node.lpt1 {
                        PrintHandle::LPrint
                    } else {
                        PrintHandle::Print
                    },
                    pos,
                );
            }
        }
    }

    fn generate_opt_format_string_instructions(&mut self, print_node: &PrintNode, pos: Location) {
        match &print_node.format_string {
            Some(format_string) => {
                self.generate_expression_instructions(format_string.clone());
                self.push(Instruction::PushUnnamed, pos);
            }
            None => {
                self.push_load_unnamed_arg("", pos);
            }
        }
    }

    fn generate_print_arg_instructions(&mut self, print_arg: PrintArg, pos: Location) {
        match print_arg {
            PrintArg::Expression(Locatable { element: arg, pos }) => {
                self.push_load_unnamed_arg(PrintArgType::Expression, pos);
                self.generate_expression_instructions(arg.at(pos));
                self.push(Instruction::PushUnnamed, pos);
            }
            PrintArg::Comma => {
                self.push_load_unnamed_arg(PrintArgType::Comma, pos);
            }
            PrintArg::Semicolon => {
                self.push_load_unnamed_arg(PrintArgType::Semicolon, pos);
            }
        }
    }
}
