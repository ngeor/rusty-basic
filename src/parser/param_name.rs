use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::name::MAX_LENGTH;
use crate::parser::pc::common::*;
use crate::parser::pc::map::and_then;
use crate::parser::pc::*;
use crate::parser::pc2::unary_fn::UnaryFnParser;
use crate::parser::pc2::Parser;
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use std::io::BufRead;
use std::str::FromStr;

// Parses a Param name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType

pub fn param_name_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ParamNameNode, QError>> {
    and_then(
        opt_seq2(
            with_pos(param_name_p().convert_to_fn()),
            type_definition_extended(),
        ),
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
        },
    )
}

fn param_name_p<R>() -> impl Parser<R, Output = (Name, bool)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
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

fn type_definition_extended<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ParamType, QError>> {
    // <ws+> AS <ws+> identifier
    drop_left(crate::parser::pc::ws::seq2(
        crate::parser::pc::ws::one_or_more_leading(keyword(Keyword::As)),
        demand(
            extended_type(),
            QError::syntax_error_fn("Expected: type after AS"),
        ),
        QError::syntax_error_fn("Expected: whitespace after AS"),
    ))
}

fn extended_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ParamType, QError>> {
    and_then(
        with_pos(any_identifier_without_dot()),
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
