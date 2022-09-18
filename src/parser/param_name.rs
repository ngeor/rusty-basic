use std::str::FromStr;

use crate::common::*;
use crate::parser::base::and_pc::{AndDemandTrait, AndTrait};
use crate::parser::base::and_then_pc::AndThenTrait;
use crate::parser::base::parsers::{AndOptTrait, KeepRightTrait, Parser};
use crate::parser::expression;
use crate::parser::name::MAX_LENGTH;
use crate::parser::specific::{
    identifier_without_dot_p, keyword_followed_by_whitespace_p, whitespace, OrSyntaxErrorTrait,
    WithPosTrait,
};
use crate::parser::types::*;

// Parses a Param name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType

pub fn param_name_node_p() -> impl Parser<Output = ParamNameNode> {
    param_name_p()
        .with_pos()
        .and_opt(type_definition_extended_p())
        .and_then(
            |(
                Locatable {
                    element: (name, is_array),
                    pos,
                },
                opt_type_definition,
            )| match name {
                Name::Bare(b) => match opt_type_definition {
                    Some(param_type) => match param_type {
                        ParamType::UserDefined(_) => {
                            if b.contains('.') {
                                Err(QError::IdentifierCannotIncludePeriod)
                            } else {
                                Ok(ParamName::new(b, final_param_type(param_type, is_array))
                                    .at(pos))
                            }
                        }
                        _ => Ok(ParamName::new(b, final_param_type(param_type, is_array)).at(pos)),
                    },
                    None => {
                        Ok(ParamName::new(b, final_param_type(ParamType::Bare, is_array)).at(pos))
                    }
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
            },
        )
}

fn param_name_p() -> impl Parser<Output = (Name, bool)> {
    expression::word::word_p().and_then(|name_expr| match name_expr {
        Expression::Variable(var_name, _) => Ok((var_name, false)),
        Expression::Property(_, _, _) => {
            // only allowed if we can fold it back into a single name
            name_expr
                .fold_name()
                .ok_or(QError::syntax_error("Invalid parameter name"))
                .map(|x| (x, false))
        }
        Expression::FunctionCall(var_name, args) => {
            if args.is_empty() {
                Ok((var_name, true))
            } else {
                Err(QError::syntax_error("Invalid parameter name"))
            }
        }
        _ => Err(QError::syntax_error("Invalid parameter name")),
    })
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
    whitespace()
        .and(keyword_followed_by_whitespace_p(Keyword::As))
        .and_demand(extended_type_p().or_syntax_error("Expected: type after AS"))
        .keep_right()
}

fn extended_type_p() -> impl Parser<Output = ParamType> {
    identifier_without_dot_p()
        .with_pos()
        .and_then(
            |Locatable { element: x, pos }| match Keyword::from_str(&x) {
                Ok(Keyword::Single) => Ok(ParamType::BuiltIn(
                    TypeQualifier::BangSingle,
                    BuiltInStyle::Extended,
                )),
                Ok(Keyword::Double) => Ok(ParamType::BuiltIn(
                    TypeQualifier::HashDouble,
                    BuiltInStyle::Extended,
                )),
                Ok(Keyword::String_) => Ok(ParamType::BuiltIn(
                    TypeQualifier::DollarString,
                    BuiltInStyle::Extended,
                )),
                Ok(Keyword::Integer) => Ok(ParamType::BuiltIn(
                    TypeQualifier::PercentInteger,
                    BuiltInStyle::Extended,
                )),
                Ok(Keyword::Long) => Ok(ParamType::BuiltIn(
                    TypeQualifier::AmpersandLong,
                    BuiltInStyle::Extended,
                )),
                Ok(_) => Err(QError::syntax_error(
                    "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
                )),
                Err(_) => {
                    if x.len() > MAX_LENGTH {
                        Err(QError::IdentifierTooLong)
                    } else {
                        let type_name: BareName = x.into();
                        Ok(ParamType::UserDefined(type_name.at(pos)))
                    }
                }
            },
        )
}
