use std::str::FromStr;

use crate::common::*;
use crate::parser::base::and_pc::AndDemandTrait;
use crate::parser::base::and_then_pc::AndThenTrait;
use crate::parser::base::parsers::{AndOptTrait, FnMapTrait, HasOutput, KeepRightTrait, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::name::name_with_dot_p;
use crate::parser::specific::csv::csv_one_or_more;
use crate::parser::specific::in_parenthesis::in_parenthesis_non_opt;
use crate::parser::specific::whitespace::WhitespaceTrait;
use crate::parser::specific::with_pos::WithPosTrait;
use crate::parser::specific::{
    identifier_or_keyword_without_dot, item_p, keyword, keyword_followed_by_whitespace_p,
    OrErrorTrait,
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

pub fn dim_name_node_p() -> impl Parser<Output = DimNameNode> {
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

pub fn redim_name_node_p() -> impl Parser<Output = DimNameNode> {
    name_with_dot_p()
        .with_pos()
        .and_demand(array_dimensions_p().or_syntax_error("Expected: array dimensions"))
        .and_opt(type_definition_extended_p())
        .and_then(
            |((name_node, array_dimensions), opt_extended_type_definition)| {
                map_name_opt_extended_type_definition(
                    name_node,
                    Some(array_dimensions),
                    opt_extended_type_definition,
                )
            },
        )
}

fn array_dimensions_p() -> impl Parser<Output = ArrayDimensions> {
    in_parenthesis_non_opt(
        csv_one_or_more(array_dimension_p()).or_syntax_error("Expected: array dimension"),
    )
}

fn array_dimension_p() -> impl Parser<Output = ArrayDimension> {
    expression::expression_node_p()
        .and_opt(
            keyword(Keyword::To)
                .preceded_by_req_ws()
                .and_demand(
                    expression::guarded_expression_node_p()
                        .or_syntax_error("Expected: expression after TO"),
                )
                .keep_right(),
        )
        .fn_map(|(l, opt_r)| match opt_r {
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

fn type_definition_extended_p() -> impl Parser<Output = DimType> {
    // <ws+> AS <ws+> identifier
    keyword_followed_by_whitespace_p(Keyword::As)
        .preceded_by_req_ws()
        .and_demand(extended_type_p().or_syntax_error("Expected: type after AS"))
        .keep_right()
}

fn extended_type_p() -> impl Parser<Output = DimType> {
    ExtendedTypeParser {}
}

struct ExtendedTypeParser;

impl HasOutput for ExtendedTypeParser {
    type Output = DimType;
}

impl Parser for ExtendedTypeParser {
    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_identifier = identifier_or_keyword_without_dot()
            .with_pos()
            .parse(reader)?;
        match opt_identifier {
            Some(Locatable { element: x, pos }) => match Keyword::from_str(&x.text) {
                Ok(Keyword::Single) => Self::built_in(TypeQualifier::BangSingle),
                Ok(Keyword::Double) => Self::built_in(TypeQualifier::HashDouble),
                Ok(Keyword::String_) => Self::string(reader),
                Ok(Keyword::Integer) => Self::built_in(TypeQualifier::PercentInteger),
                Ok(Keyword::Long) => Self::built_in(TypeQualifier::AmpersandLong),
                Ok(_) => Err(QError::syntax_error(
                    "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
                )),
                Err(_) => {
                    if x.text.len() > name::MAX_LENGTH {
                        Err(QError::IdentifierTooLong)
                    } else {
                        Ok(Some(DimType::UserDefined(BareName::from(x.text).at(pos))))
                    }
                }
            },
            _ => Ok(None),
        }
    }
}

impl ExtendedTypeParser {
    fn built_in(q: TypeQualifier) -> Result<Option<DimType>, QError> {
        Ok(Some(DimType::BuiltIn(q, BuiltInStyle::Extended)))
    }

    fn string(reader: &mut impl Tokenizer) -> Result<Option<DimType>, QError> {
        let opt_len = item_p('*')
            .surrounded_by_opt_ws()
            .and_demand(
                expression::expression_node_p().or_syntax_error("Expected: string length after *"),
            )
            .keep_right()
            .parse(reader)?;
        match opt_len {
            Some(len) => Ok(Some(DimType::FixedLengthString(len, 0))),
            _ => Self::built_in(TypeQualifier::DollarString),
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
