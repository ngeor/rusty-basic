use crate::common::*;
use crate::parser::expression::expression_node_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;

// Parses a Param name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType

pub fn param_name_node_p() -> impl Parser<Output = ParamNameNode> {
    param_name_p()
        .and_opt(type_definition_extended_p())
        .and_then(|((name, is_array, pos), opt_type_definition)| match name {
            Name::Bare(b) => match opt_type_definition {
                Some(param_type) => match param_type {
                    ParamType::UserDefined(_) => {
                        if b.contains('.') {
                            Err(QError::IdentifierCannotIncludePeriod)
                        } else {
                            Ok(ParamName::new(b, final_param_type(param_type, is_array)).at(pos))
                        }
                    }
                    _ => Ok(ParamName::new(b, final_param_type(param_type, is_array)).at(pos)),
                },
                None => Ok(ParamName::new(b, final_param_type(ParamType::Bare, is_array)).at(pos)),
            },
            Name::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) => match opt_type_definition {
                Some(_) => Err(QError::syntax_error(
                    "Identifier cannot end with %, &, !, #, or $",
                )),
                None => Ok(ParamName::new(
                    bare_name,
                    final_param_type(
                        ParamType::BuiltIn(qualifier, BuiltInStyle::Compact),
                        is_array,
                    ),
                )
                .at(pos)),
            },
        })
}

fn param_name_p() -> impl Parser<Output = (Name, bool, Location)> {
    expression_node_p().and_then(
        |Locatable {
             element: name_expr,
             pos,
         }| match name_expr {
            Expression::Variable(var_name, _) => Ok((var_name, false, pos)),
            Expression::Property(_, _, _) => {
                // only allowed if we can fold it back into a single name
                name_expr
                    .fold_name()
                    .ok_or(QError::syntax_error("Invalid parameter name"))
                    .map(|x| (x, false, pos))
            }
            Expression::FunctionCall(var_name, args) => {
                if args.is_empty() {
                    Ok((var_name, true, pos))
                } else {
                    Err(QError::syntax_error("Invalid parameter name"))
                }
            }
            _ => Err(QError::syntax_error("Invalid parameter name")),
        },
    )
}

fn final_param_type(param_type: ParamType, is_array: bool) -> ParamType {
    if is_array {
        ParamType::Array(Box::new(param_type))
    } else {
        param_type
    }
}

fn type_definition_extended_p() -> impl Parser<Output = ParamType> {
    // <ws+> AS <ws+> identifier
    seq3(
        whitespace().and(keyword(Keyword::As)),
        whitespace().no_incomplete(),
        extended_type_p().or_syntax_error(
            "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
        ),
        |_, _, param_type| param_type,
    )
}

fn extended_type_p() -> impl Parser<Output = ParamType> {
    Alt2::new(
        identifier_with_dots().with_pos().and_then(
            |Locatable {
                 element: token,
                 pos,
             }| {
                if token.text.contains('.') {
                    Err(QError::IdentifierCannotIncludePeriod)
                } else {
                    Ok(ParamType::UserDefined(BareName::new(token.text).at(pos)))
                }
            },
        ),
        keyword_map(&[
            (
                Keyword::Single,
                ParamType::BuiltIn(TypeQualifier::BangSingle, BuiltInStyle::Extended),
            ),
            (
                Keyword::Double,
                ParamType::BuiltIn(TypeQualifier::HashDouble, BuiltInStyle::Extended),
            ),
            (
                Keyword::String,
                ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Extended),
            ),
            (
                Keyword::Integer,
                ParamType::BuiltIn(TypeQualifier::PercentInteger, BuiltInStyle::Extended),
            ),
            (
                Keyword::Long,
                ParamType::BuiltIn(TypeQualifier::AmpersandLong, BuiltInStyle::Extended),
            ),
        ]),
    )
}
