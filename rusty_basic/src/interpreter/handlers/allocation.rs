use crate::RuntimeError;
use rusty_common::{CaseInsensitiveString, NoPosIterTrait, Positioned};
use rusty_parser::specific::{
    BareName, ElementType, ExpressionType, TypeQualifier, UserDefinedTypes,
};
use rusty_variant::{UserDefinedTypeValue, VArray, Variant};

/// TODO add unit tests

/// Creates the default variant for the given type.
pub fn allocate_built_in(type_qualifier: TypeQualifier) -> Variant {
    match type_qualifier {
        TypeQualifier::BangSingle => Variant::VSingle(0.0),
        TypeQualifier::HashDouble => Variant::VDouble(0.0),
        TypeQualifier::DollarString => Variant::VString(String::new()),
        TypeQualifier::PercentInteger => Variant::VInteger(0),
        TypeQualifier::AmpersandLong => Variant::VLong(0),
    }
}

/// Allocates a new Variant that holds a string.
/// The string is padded with whitespace of the given length.
pub fn allocate_fixed_length_string(len: usize) -> Variant {
    Variant::VString(" ".repeat(len))
}

pub fn allocate_array(
    dimension_args: Vec<i32>,
    element_type: &ExpressionType,
    types: &UserDefinedTypes,
) -> Result<Variant, RuntimeError> {
    let dimensions = to_dimensions(dimension_args)?;
    Ok(Variant::VArray(Box::new(VArray::new(
        dimensions,
        allocate_array_element(element_type, types),
    ))))
}

fn to_dimensions(dimension_args: Vec<i32>) -> Result<Vec<(i32, i32)>, RuntimeError> {
    let mut dimensions: Vec<(i32, i32)> = vec![];
    let mut i: usize = 0;
    while i < dimension_args.len() {
        let lbound = dimension_args[i];
        i += 1;
        let ubound = dimension_args[i];
        if ubound < lbound {
            return Err(RuntimeError::SubscriptOutOfRange);
        }
        i += 1;
        dimensions.push((lbound, ubound));
    }
    Ok(dimensions)
}

pub fn allocate_user_defined_type(
    user_defined_type_name: &BareName,
    types: &UserDefinedTypes,
) -> Variant {
    Variant::VUserDefined(Box::new(allocate_user_defined_type_inner(
        user_defined_type_name,
        types,
    )))
}

fn allocate_array_element(element_type: &ExpressionType, types: &UserDefinedTypes) -> Variant {
    match element_type {
        ExpressionType::BuiltIn(q) => allocate_built_in(*q),
        ExpressionType::FixedLengthString(len) => allocate_fixed_length_string(*len as usize),
        ExpressionType::UserDefined(type_name) => {
            Variant::VUserDefined(Box::new(allocate_user_defined_type_inner(type_name, types)))
        }
        ExpressionType::Unresolved => panic!("Unresolved array element type"),
        ExpressionType::Array(_) => panic!("Nested arrays are not supported"),
    }
}

fn allocate_user_defined_type_inner(
    type_name: &CaseInsensitiveString,
    types: &UserDefinedTypes,
) -> UserDefinedTypeValue {
    let user_defined_type = types.get(type_name).expect("Could not find type");
    let arr: Vec<(CaseInsensitiveString, Variant)> = user_defined_type
        .elements()
        .no_pos()
        .map(|element| {
            (
                element.name.clone(),
                allocate_element_type(&element.element_type, types),
            )
        })
        .collect();
    UserDefinedTypeValue::new(arr)
}

fn allocate_element_type(element_type: &ElementType, types: &UserDefinedTypes) -> Variant {
    match element_type {
        ElementType::Single => allocate_built_in(TypeQualifier::BangSingle),
        ElementType::Double => allocate_built_in(TypeQualifier::HashDouble),
        ElementType::FixedLengthString(_, len) => allocate_fixed_length_string(*len as usize),
        ElementType::Integer => allocate_built_in(TypeQualifier::PercentInteger),
        ElementType::Long => allocate_built_in(TypeQualifier::AmpersandLong),
        ElementType::UserDefined(Positioned { element, .. }) => {
            Variant::VUserDefined(Box::new(allocate_user_defined_type_inner(element, types)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_string() {
        assert_eq!(
            allocate_fixed_length_string(1),
            Variant::VString(" ".to_owned())
        );
        assert_eq!(
            allocate_fixed_length_string(2),
            Variant::VString("  ".to_owned())
        );
    }
}
