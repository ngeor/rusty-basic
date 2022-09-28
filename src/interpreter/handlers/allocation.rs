use crate::common::{QError, StringUtils};
use crate::interpreter::arguments::ArgumentInfo;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::{BareName, ExpressionType, TypeQualifier, UserDefinedTypes};
use crate::variant::{QBNumberCast, UserDefinedTypeValue, VArray, Variant};

pub fn allocate_built_in<T: InterpreterTrait>(
    interpreter: &mut T,
    q: TypeQualifier,
) -> Result<(), QError> {
    let v = Variant::from(q);
    interpreter.registers_mut().set_a(v);
    Ok(())
}

pub fn allocate_fixed_length_string<T: InterpreterTrait>(
    interpreter: &mut T,
    len: u16,
) -> Result<(), QError> {
    let s: String = (0..len).map(|_| ' ').collect();
    let v = Variant::VString(s);
    interpreter.registers_mut().set_a(v);
    Ok(())
}

pub fn allocate_array<T: InterpreterTrait>(
    interpreter: &mut T,
    element_type: &ExpressionType,
) -> Result<(), QError> {
    let arguments = interpreter
        .context_mut()
        .drop_arguments_for_array_allocation();
    let r_args: Result<Vec<i32>, QError> = arguments
        .iter()
        .map(|ArgumentInfo { value, .. }| value)
        .map(QBNumberCast::try_cast)
        .collect();
    let args = r_args?;
    let mut dimensions: Vec<(i32, i32)> = vec![];
    let mut i: usize = 0;
    while i < args.len() {
        let lbound = args[i];
        i += 1;
        let ubound = args[i];
        if ubound < lbound {
            return Err(QError::SubscriptOutOfRange);
        }
        i += 1;
        dimensions.push((lbound, ubound));
    }
    let array = Variant::VArray(Box::new(VArray::new(
        dimensions,
        allocate_array_element(element_type, interpreter.user_defined_types()),
    )));
    interpreter.registers_mut().set_a(array);
    Ok(())
}

pub fn allocate_user_defined_type<T: InterpreterTrait>(
    interpreter: &mut T,
    user_defined_type_name: &BareName,
) -> Result<(), QError> {
    let v: Variant = Variant::VUserDefined(Box::new(UserDefinedTypeValue::new(
        user_defined_type_name,
        interpreter.user_defined_types(),
    )));
    interpreter.registers_mut().set_a(v);
    Ok(())
}

fn allocate_array_element(element_type: &ExpressionType, types: &UserDefinedTypes) -> Variant {
    match element_type {
        ExpressionType::BuiltIn(q) => Variant::from(*q),
        ExpressionType::FixedLengthString(len) => "".fix_length(*len as usize).into(),
        ExpressionType::UserDefined(type_name) => {
            Variant::VUserDefined(Box::new(UserDefinedTypeValue::new(type_name, types)))
        }
        ExpressionType::Unresolved => panic!("Unresolved array element type"),
        ExpressionType::Array(_) => panic!("Nested arrays are not supported"),
    }
}
