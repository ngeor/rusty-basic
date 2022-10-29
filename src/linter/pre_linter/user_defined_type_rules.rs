use crate::common::{
    AtLocation, Locatable, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError,
};
use crate::linter::pre_linter::ConstantMap;
use crate::parser::{
    BareName, Element, ElementNode, ElementType, Expression, ExpressionNode, TypeQualifier,
    UserDefinedType, UserDefinedTypes,
};
use crate::variant::Variant;
use std::collections::HashMap;

pub fn user_defined_type(
    user_defined_types: &mut UserDefinedTypes,
    global_constants: &ConstantMap,
    user_defined_type: &UserDefinedType,
) -> Result<(), QErrorNode> {
    let type_name: &BareName = user_defined_type.bare_name();
    if user_defined_types.contains_key(type_name) {
        // duplicate type definition
        Err(QError::DuplicateDefinition).with_err_no_pos()
    } else {
        let mut resolved_elements: HashMap<BareName, ElementType> = HashMap::new();
        for Locatable {
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
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let resolved_element_type = match element_type {
                ElementType::Integer => ElementType::Integer,
                ElementType::Long => ElementType::Long,
                ElementType::Single => ElementType::Single,
                ElementType::Double => ElementType::Double,
                ElementType::FixedLengthString(str_len_expression_node, _) => {
                    let l: u16 =
                        validate_element_type_str_len(global_constants, str_len_expression_node)?;
                    ElementType::FixedLengthString(
                        Expression::IntegerLiteral(l as i32).at(str_len_expression_node),
                        l,
                    )
                }
                ElementType::UserDefined(Locatable {
                    element: referred_name,
                    pos,
                }) => {
                    if !user_defined_types.contains_key(referred_name) {
                        return Err(QError::TypeNotDefined).with_err_at(pos);
                    }
                    ElementType::UserDefined(referred_name.clone().at(pos))
                }
            };
            resolved_elements.insert(element_name.clone(), resolved_element_type);
        }
        let mut elements: Vec<ElementNode> = vec![];
        for Locatable {
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

fn validate_element_type_str_len(
    global_constants: &ConstantMap,
    str_len_expression_node: &ExpressionNode,
) -> Result<u16, QErrorNode> {
    let Locatable {
        element: str_len_expression,
        pos,
    } = str_len_expression_node;
    match str_len_expression {
        Expression::IntegerLiteral(i) => {
            // parser already covers that i is between 1..MAX_INT
            Ok(*i as u16)
        }
        Expression::Variable(name_expr, _) => {
            // only constants allowed
            if let Some(qualifier) = name_expr.qualifier() {
                match global_constants.get(name_expr.bare_name()) {
                    // constant exists
                    Some(const_value) => {
                        match const_value {
                            Variant::VInteger(i) => {
                                if qualifier == TypeQualifier::PercentInteger
                                    && *i >= 1
                                    && *i <= crate::variant::MAX_INTEGER
                                {
                                    Ok(*i as u16)
                                } else {
                                    // illegal string length or using wrong qualifier to reference the int constant
                                    Err(QError::InvalidConstant).with_err_at(pos)
                                }
                            }
                            _ => {
                                // only integer constants allowed
                                Err(QError::InvalidConstant).with_err_at(pos)
                            }
                        }
                    }
                    // constant does not exist
                    None => Err(QError::InvalidConstant).with_err_at(pos),
                }
            } else {
                // bare name constant
                match global_constants.get(name_expr.bare_name()) {
                    // constant exists
                    Some(const_value) => {
                        match const_value {
                            Variant::VInteger(i) => {
                                if *i >= 1 && *i <= crate::variant::MAX_INTEGER {
                                    Ok(*i as u16)
                                } else {
                                    // illegal string length
                                    Err(QError::InvalidConstant).with_err_at(pos)
                                }
                            }
                            _ => {
                                // only integer constants allowed
                                Err(QError::InvalidConstant).with_err_at(pos)
                            }
                        }
                    }
                    // constant does not exist
                    None => Err(QError::InvalidConstant).with_err_at(pos),
                }
            }
        }
        _ => panic!("Unexpected string length {:?}", str_len_expression),
    }
}
