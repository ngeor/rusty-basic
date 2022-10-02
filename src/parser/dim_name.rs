use std::str::FromStr;

use crate::common::*;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::name::name_with_dot_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
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
    Seq3::new(
        name_with_dot_p().with_pos(),
        array_dimensions_p().allow_default(),
        type_definition_extended_p().allow_none(),
    )
    .and_then(
        |(name_node, array_dimensions, opt_extended_type_definition)| {
            map_name_opt_extended_type_definition(
                name_node,
                array_dimensions,
                opt_extended_type_definition,
            )
        },
    )
}

pub fn redim_name_node_p() -> impl Parser<Output = DimNameNode> {
    Seq3::new(
        name_with_dot_p().with_pos(),
        array_dimensions_p().or_syntax_error("Expected: array dimensions"),
        type_definition_extended_p().allow_none(),
    )
    .and_then(
        |(name_node, array_dimensions, opt_extended_type_definition)| {
            map_name_opt_extended_type_definition(
                name_node,
                array_dimensions,
                opt_extended_type_definition,
            )
        },
    )
}

fn array_dimensions_p() -> impl Parser<Output = ArrayDimensions> {
    in_parenthesis(csv(array_dimension_p()).or_syntax_error("Expected: array dimension"))
}

// expr (e.g. 10)
// expr ws+ TO ws+ expr (e.g. 1 TO 10)
// paren_expr ws* TO ws* paren_expr
fn array_dimension_p() -> impl Parser<Output = ArrayDimension> {
    expression::expression_node_p()
        .and_opt_factory(|lower_bound_expr| {
            whitespace_boundary_after_expr(lower_bound_expr)
                .and(keyword(Keyword::To))
                .then_demand(
                    expression::guarded_expression_node_p()
                        .or_syntax_error("Expected: expression after TO"),
                )
        })
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

fn type_definition_extended_p() -> impl Parser<Output = DimType> {
    // <ws+> AS <ws+> identifier
    seq3(
        whitespace().and(keyword(Keyword::As)),
        whitespace().no_incomplete(),
        extended_type_p().or_syntax_error("Expected: type after AS"),
        |_, _, identifier| identifier,
    )
}

fn extended_type_p() -> impl Parser<Output = DimType> {
    ExtendedTypeParser {}
}

struct ExtendedTypeParser;

impl Parser for ExtendedTypeParser {
    type Output = DimType;
    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        // TODO rewrite this
        // TODO only try to parse keyword if the token kind matches
        let Locatable { element: x, pos } = identifier_or_keyword_without_dot()
            .with_pos()
            .parse(reader)?;
        match Keyword::from_str(&x.text) {
            Ok(Keyword::Single) => Self::built_in(TypeQualifier::BangSingle),
            Ok(Keyword::Double) => Self::built_in(TypeQualifier::HashDouble),
            Ok(Keyword::String_) => Self::string(reader),
            Ok(Keyword::Integer) => Self::built_in(TypeQualifier::PercentInteger),
            Ok(Keyword::Long) => Self::built_in(TypeQualifier::AmpersandLong),
            Ok(_) => Err(QError::syntax_error(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
            )),
            Err(_) => {
                if x.text.chars().count() > name::MAX_LENGTH {
                    Err(QError::IdentifierTooLong)
                } else {
                    Ok(DimType::UserDefined(BareName::from(x.text).at(pos)))
                }
            }
        }
    }
}

impl ExtendedTypeParser {
    fn built_in(q: TypeQualifier) -> Result<DimType, QError> {
        Ok(DimType::BuiltIn(q, BuiltInStyle::Extended))
    }

    fn string(reader: &mut impl Tokenizer) -> Result<DimType, QError> {
        let opt_len = star()
            .then_demand(
                expression::expression_node_p().or_syntax_error("Expected: string length after *"),
            )
            .parse_opt(reader)?;
        match opt_len {
            Some(len) => Ok(DimType::FixedLengthString(len, 0)),
            _ => Self::built_in(TypeQualifier::DollarString),
        }
    }
}

fn map_name_opt_extended_type_definition(
    name_node: NameNode,
    array_dimensions: ArrayDimensions,
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
    let final_dim_type = if array_dimensions.is_empty() {
        dim_type
    } else {
        DimType::Array(array_dimensions, Box::new(dim_type))
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
