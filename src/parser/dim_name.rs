use std::str::FromStr;

use crate::common::*;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::name::name_with_dot_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::{
    identifier_without_dot_p, in_parenthesis_p, keyword_followed_by_whitespace_p, keyword_p,
    PcSpecific,
};
use crate::parser::types::*;

// Parses a declared name. Possible options:
// A
// A%
// A AS INTEGER
// A AS UserDefinedType
//
// Arrays:
// A(10)
// A$(1 TO 2, 0 TO 10)
// A(1 TO 5) AS INTEGER

pub fn dim_name_node_p<R>() -> impl Parser<R, Output = DimNameNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    name_with_dot_p()
        .with_pos()
        .and_opt(array_dimensions_p())
        .and_opt(type_definition_extended_p())
        .and_then(
            |((name_node, opt_array_dimensions), opt_extended_type_definition)| {
                map_name_opt_extended_type_definition(
                    name_node,
                    opt_array_dimensions,
                    opt_extended_type_definition,
                )
            },
        )
}

fn array_dimensions_p<R>() -> impl Parser<R, Output = ArrayDimensions>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    in_parenthesis_p(
        array_dimension_p()
            .csv()
            .or_syntax_error("Expected: array dimension"),
    )
}

fn array_dimension_p<R>() -> impl Parser<R, Output = ArrayDimension>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression::expression_node_p()
        .and_opt(
            whitespace_p()
                .and(keyword_p(Keyword::To))
                .and_demand(
                    expression::guarded_expression_node_p()
                        .or_syntax_error("Expected: expression after TO"),
                )
                .keep_right(),
        )
        .map(|(l, opt_r)| match opt_r {
            Some(r) => ArrayDimension {
                lbound: Some(l),
                ubound: r,
            },
            None => ArrayDimension {
                lbound: None,
                ubound: l,
            },
        })
}

fn type_definition_extended_p<R>() -> impl Parser<R, Output = DimType>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    // <ws+> AS <ws+> identifier
    whitespace_p()
        .and(keyword_followed_by_whitespace_p(Keyword::As))
        .and_demand(extended_type_p().or_syntax_error("Expected: type after AS"))
        .keep_right()
}

fn extended_type_p<R>() -> impl Parser<R, Output = DimType>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    ExtendedTypeParser {}
}

struct ExtendedTypeParser;

impl<R> Parser<R> for ExtendedTypeParser
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    type Output = DimType;

    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_identifier) = identifier_without_dot_p().with_pos().parse(reader)?;
        match opt_identifier {
            Some(Locatable { element: x, pos }) => match Keyword::from_str(x.as_str()) {
                Ok(Keyword::Single) => Self::built_in(reader, TypeQualifier::BangSingle),
                Ok(Keyword::Double) => Self::built_in(reader, TypeQualifier::HashDouble),
                Ok(Keyword::String_) => Self::string(reader),
                Ok(Keyword::Integer) => Self::built_in(reader, TypeQualifier::PercentInteger),
                Ok(Keyword::Long) => Self::built_in(reader, TypeQualifier::AmpersandLong),
                Ok(_) => Err((
                    reader,
                    QError::syntax_error(
                        "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
                    ),
                )),
                Err(_) => {
                    if x.len() > name::MAX_LENGTH {
                        Err((reader, QError::IdentifierTooLong))
                    } else {
                        Ok((
                            reader,
                            Some(DimType::UserDefined(BareName::from(x).at(pos))),
                        ))
                    }
                }
            },
            _ => Ok((reader, None)),
        }
    }
}

impl ExtendedTypeParser {
    fn built_in<R>(reader: R, q: TypeQualifier) -> ReaderResult<R, DimType, R::Err>
    where
        R: Reader,
    {
        Ok((reader, Some(DimType::BuiltIn(q, BuiltInStyle::Extended))))
    }

    fn string<R>(reader: R) -> ReaderResult<R, DimType, R::Err>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        let (reader, opt_len) = item_p('*')
            .surrounded_by_opt_ws()
            .and_demand(
                expression::expression_node_p().or_syntax_error("Expected: string length after *"),
            )
            .keep_right()
            .parse(reader)?;
        match opt_len {
            Some(len) => Ok((reader, Some(DimType::FixedLengthString(len, 0)))),
            _ => Self::built_in(reader, TypeQualifier::DollarString),
        }
    }
}

fn map_name_opt_extended_type_definition(
    name_node: NameNode,
    opt_array_dimensions: Option<ArrayDimensions>,
    opt_type_definition: Option<DimType>,
) -> Result<DimNameNode, QError> {
    let Locatable { element: name, pos } = name_node;
    let dim_type: DimType = match &name {
        Name::Bare(bare_name) => {
            map_bare_name_opt_extended_type_definition(bare_name, opt_type_definition)?
        }
        Name::Qualified(qualified_name) => {
            map_qualified_name_opt_extended_type_definition(qualified_name, opt_type_definition)?
        }
    };
    let final_dim_type = match opt_array_dimensions {
        Some(array_dimensions) => DimType::Array(array_dimensions, Box::new(dim_type)),
        _ => dim_type,
    };
    let bare_name: BareName = name.into();
    let dim_name = DimName::new(bare_name, final_dim_type);
    Ok(dim_name.at(pos))
}

fn map_bare_name_opt_extended_type_definition(
    bare_name: &BareName,
    opt_type_definition: Option<DimType>,
) -> Result<DimType, QError> {
    match opt_type_definition {
        Some(dim_type) => match dim_type {
            DimType::UserDefined(_) => {
                if bare_name.contains('.') {
                    Err(QError::IdentifierCannotIncludePeriod)
                } else {
                    Ok(dim_type)
                }
            }
            _ => Ok(dim_type),
        },
        None => Ok(DimType::Bare),
    }
}

fn map_qualified_name_opt_extended_type_definition(
    qualified_name: &QualifiedName,
    opt_type_definition: Option<DimType>,
) -> Result<DimType, QError> {
    if opt_type_definition.is_some() {
        Err(QError::syntax_error(
            "Identifier cannot end with %, &, !, #, or $",
        ))
    } else {
        let QualifiedName { qualifier, .. } = qualified_name;
        Ok(DimType::BuiltIn(*qualifier, BuiltInStyle::Compact))
    }
}
