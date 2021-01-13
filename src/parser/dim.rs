use crate::common::{HasLocation, QError};
use crate::parser::pc::*;
use crate::parser::pc_specific::{keyword_followed_by_whitespace_p, PcSpecific};
use crate::parser::{dim_name, Keyword, Statement};

/// Parses DIM statement
pub fn dim_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_followed_by_whitespace_p(Keyword::Dim)
        .and_opt(keyword_followed_by_whitespace_p(Keyword::Shared))
        .and_demand(dim_name::dim_name_node_p().or_syntax_error("Expected: name after DIM"))
        .map(|((_, _opt_shared), r)| Statement::Dim(r))
}

#[cfg(test)]
mod tests {
    use crate::common::*;

    use crate::parser::test_utils::*;
    use crate::parser::types::*;

    macro_rules! assert_parse_dim_extended_built_in {
        ($name: literal, $keyword: literal, $qualifier: ident) => {
            let input = format!("DIM {} AS {}", $name, $keyword);
            let p = parse(input).demand_single_statement();
            assert_eq!(
                p,
                crate::parser::Statement::Dim(
                    crate::parser::DimName::new(
                        $name.into(),
                        crate::parser::DimType::BuiltIn(
                            TypeQualifier::$qualifier,
                            crate::parser::BuiltInStyle::Extended
                        )
                    )
                    .at_rc(1, 5)
                )
            );
        };
    }

    #[test]
    fn test_parse_dim_extended_built_in() {
        assert_parse_dim_extended_built_in!("A", "SINGLE", BangSingle);
        assert_parse_dim_extended_built_in!("A.", "SINGLE", BangSingle);
        assert_parse_dim_extended_built_in!("A.B", "SINGLE", BangSingle);
        assert_parse_dim_extended_built_in!("AB", "DOUBLE", HashDouble);
        assert_parse_dim_extended_built_in!("S", "STRING", DollarString);
        assert_parse_dim_extended_built_in!("I", "INTEGER", PercentInteger);
        assert_parse_dim_extended_built_in!("L1", "LONG", AmpersandLong);
    }

    #[test]
    fn test_parse_dim_extended_user_defined() {
        let var_names = ["A", "ABC"];
        let types = [
            "FirstName",
            "Address2",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMN", // 40 characters max length
        ];
        for var_name in &var_names {
            for var_type in &types {
                let input = format!("DIM {} AS {}", var_name, var_type);
                let p = parse(input).demand_single_statement();
                let var_name_bare: BareName = (*var_name).into();
                let var_type_bare: BareName = (*var_type).into();
                match p {
                    Statement::Dim(dim_name_node) => {
                        let Locatable {
                            element: dim_name,
                            pos,
                        } = dim_name_node;
                        assert_eq!(pos, Location::new(1, 5));
                        assert_eq!(dim_name.as_ref().clone(), var_name_bare);
                        match dim_name.dim_type() {
                            DimType::UserDefined(Locatable { element, .. }) => {
                                assert_eq!(element, &var_type_bare);
                            }
                            _ => panic!("Expected user defined type"),
                        }
                    }
                    _ => panic!("Expected dim statement"),
                }
            }
        }
    }

    #[test]
    fn test_parse_dim_extended_wrong_keyword() {
        let input = "DIM X AS AS";
        assert_eq!(
            parse_err(input),
            QError::SyntaxError(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier".to_string()
            )
        );
    }

    #[test]
    fn test_parse_dim_extended_with_qualified_name() {
        let input = "DIM A$ AS STRING";
        assert_eq!(
            parse_err(input),
            QError::syntax_error("Identifier cannot end with %, &, !, #, or $")
        );
    }

    #[test]
    fn test_parse_dim_user_defined_too_long() {
        let input = "DIM A AS ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNO";
        assert_eq!(parse_err(input), QError::IdentifierTooLong);
    }

    #[test]
    fn test_parse_dim_user_defined_cannot_include_period() {
        let input = "DIM A.B AS Card";
        assert_eq!(parse_err(input), QError::IdentifierCannotIncludePeriod);
    }

    macro_rules! assert_parse_dim_compact {
        ($name: literal) => {
            let input = format!("DIM {}", $name);
            let p = parse(input).demand_single_statement();
            assert_eq!(
                p,
                Statement::Dim(DimName::new($name.into(), DimType::Bare).at_rc(1, 5))
            );
        };

        ($name: literal, $keyword: literal, $qualifier: ident) => {
            let input = format!("DIM {}{}", $name, $keyword);
            let p = parse(input).demand_single_statement();
            assert_eq!(
                p,
                Statement::Dim(
                    DimName::new(
                        $name.into(),
                        DimType::BuiltIn(TypeQualifier::$qualifier, BuiltInStyle::Compact)
                    )
                    .at_rc(1, 5)
                )
            );
        };
    }

    #[test]
    fn test_parse_dim_compact() {
        assert_parse_dim_compact!("A");
        assert_parse_dim_compact!("A.");
        assert_parse_dim_compact!("A.B");
        assert_parse_dim_compact!("A", "!", BangSingle);
        assert_parse_dim_compact!("A.", "!", BangSingle);
        assert_parse_dim_compact!("A.B", "!", BangSingle);
        assert_parse_dim_compact!("BC", "#", HashDouble);
        assert_parse_dim_compact!("X", "$", DollarString);
        assert_parse_dim_compact!("Z", "%", PercentInteger);
        assert_parse_dim_compact!("L1", "&", AmpersandLong);
    }

    #[test]
    fn test_parse_array_single_dimension_ubound() {
        let input = "DIM A$(2)";
        let p = parse(input).demand_single_statement();
        assert_eq!(
            p,
            Statement::Dim(
                DimName::new(
                    "A".into(),
                    DimType::Array(
                        vec![ArrayDimension {
                            lbound: None,
                            ubound: 2.as_lit_expr(1, 8)
                        }],
                        Box::new(DimType::BuiltIn(
                            TypeQualifier::DollarString,
                            BuiltInStyle::Compact
                        ))
                    )
                )
                .at_rc(1, 5)
            )
        );
    }

    #[test]
    fn test_parse_array_single_dimension_lbound_ubound() {
        let input = "DIM A(1 TO 2)";
        let p = parse(input).demand_single_statement();
        assert_eq!(
            p,
            Statement::Dim(
                DimName::new(
                    "A".into(),
                    DimType::Array(
                        vec![ArrayDimension {
                            lbound: Some(1.as_lit_expr(1, 7)),
                            ubound: 2.as_lit_expr(1, 12)
                        }],
                        Box::new(DimType::Bare)
                    )
                )
                .at_rc(1, 5)
            )
        );
    }

    #[test]
    fn test_parse_array_two_dimensions() {
        let input = "DIM A(1 TO 3, 2 TO 4)";
        let p = parse(input).demand_single_statement();
        assert_eq!(
            p,
            Statement::Dim(
                DimName::new(
                    "A".into(),
                    DimType::Array(
                        vec![
                            ArrayDimension {
                                lbound: Some(1.as_lit_expr(1, 7)),
                                ubound: 3.as_lit_expr(1, 12)
                            },
                            ArrayDimension {
                                lbound: Some(2.as_lit_expr(1, 15)),
                                ubound: 4.as_lit_expr(1, 20)
                            }
                        ],
                        Box::new(DimType::Bare)
                    )
                )
                .at_rc(1, 5)
            )
        );
    }

    mod keyword_qualified_by_string_is_allowed {
        use super::*;

        #[test]
        fn test_can_assign_to_keyword_qualified_by_string() {
            let input = "DIM DIM$";
            let program = parse(input).demand_single_statement();
            assert_eq!(
                program,
                Statement::Dim(
                    DimName::new(
                        "DIM".into(),
                        DimType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                    )
                    .at_rc(1, 5)
                )
            );
        }

        #[test]
        fn test_cannot_assign_to_other_cases_of_keyword() {
            let left_sides = ["DIM", "DIM%", "DIM&", "DIM!", "DIM#"];
            for left_side in &left_sides {
                let input = format!("DIM {}", left_side);
                assert!(matches!(parse_err(input), QError::SyntaxError(_)));
            }
        }
    }
}
