use crate::core::ValidateStringLength;
use crate::core::{LintError, LintErrorPos};
use crate::pre_linter::ConstantMap;
use rusty_common::{AtPos, Position, Positioned};
use rusty_parser::{
    BareName, Element, ElementPos, ElementType, Expression, UserDefinedType, UserDefinedTypes,
};
use std::collections::HashMap;

pub fn user_defined_type(
    user_defined_types: &mut UserDefinedTypes,
    global_constants: &ConstantMap,
    user_defined_type: &UserDefinedType,
    pos: Position,
) -> Result<(), LintErrorPos> {
    let type_name: &BareName = user_defined_type.bare_name();
    if user_defined_types.contains_key(type_name) {
        // duplicate type definition
        Err(LintError::DuplicateDefinition.at_pos(pos))
    } else {
        let mut resolved_elements: HashMap<BareName, ElementType> = HashMap::new();
        for Positioned {
            element:
                Element {
                    name: element_name,
                    element_type,
                    ..
                },
            pos,
        } in user_defined_type.elements()
        {
            if resolved_elements.contains_key(element_name) {
                // duplicate element name within type
                return Err(LintError::DuplicateDefinition.at(pos));
            }
            let resolved_element_type = match element_type {
                ElementType::Integer => ElementType::Integer,
                ElementType::Long => ElementType::Long,
                ElementType::Single => ElementType::Single,
                ElementType::Double => ElementType::Double,
                ElementType::FixedLengthString(str_len_expression_pos, _) => {
                    let l: u16 = str_len_expression_pos.validate_string_length(global_constants)?;
                    ElementType::FixedLengthString(
                        Expression::IntegerLiteral(l as i32).at(str_len_expression_pos),
                        l,
                    )
                }
                ElementType::UserDefined(Positioned {
                    element: referred_name,
                    pos,
                }) => {
                    if !user_defined_types.contains_key(referred_name) {
                        return Err(LintError::TypeNotDefined.at(pos));
                    }
                    ElementType::UserDefined(referred_name.clone().at(pos))
                }
            };
            resolved_elements.insert(element_name.clone(), resolved_element_type);
        }
        let mut elements: Vec<ElementPos> = vec![];
        for Positioned {
            element: Element { name, .. },
            pos,
        } in user_defined_type.elements()
        {
            let converted_element_type = resolved_elements.remove(name).unwrap();
            elements.push(Element::new(name.clone(), converted_element_type, vec![]).at(pos));
        }
        user_defined_types.insert(
            type_name.clone(),
            UserDefinedType::new(type_name.clone(), vec![], elements),
        );
        Ok(())
    }
}
