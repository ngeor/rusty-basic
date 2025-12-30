use rusty_pc::*;

use crate::input::RcStringView;
use crate::specific::core::dim_name::{dim_var_pos_p, redim_var_pos_p};
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::ParseError;

/// Parses DIM statement
pub fn dim_p() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq4(
        keyword(Keyword::Dim),
        whitespace(),
        opt_shared_keyword(),
        csv_non_opt(dim_var_pos_p(), "Expected: name after DIM"),
        |_, _, opt_shared, variables| {
            Statement::Dim(DimList {
                shared: opt_shared.is_some(),
                variables,
            })
        },
    )
}

/// Parses REDIM statement
pub fn redim_p() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq4(
        keyword(Keyword::Redim),
        whitespace(),
        opt_shared_keyword(),
        csv_non_opt(redim_var_pos_p(), "Expected: name after REDIM"),
        |_, _, opt_shared, variables| {
            Statement::Redim(DimList {
                shared: opt_shared.is_some(),
                variables,
            })
        },
    )
}

fn opt_shared_keyword(
) -> impl Parser<RcStringView, Output = Option<(Token, Token)>, Error = ParseError> {
    Seq2::new(keyword(Keyword::Shared), whitespace()).to_option()
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::error::ParseError;
    use crate::specific::*;
    use crate::test_utils::*;
    use crate::{
        assert_parse_dim_compact, assert_parse_dim_extended_built_in, assert_parser_err, *
    };
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
                let p = parse(&input).demand_single_statement();
                let var_name_bare: BareName = (*var_name).into();
                let var_type_bare: BareName = (*var_type).into();
                match p {
                    Statement::Dim(mut dim_list) => {
                        let Positioned {
                            element: dim_name,
                            pos,
                        } = dim_list.variables.pop().unwrap();
                        assert_eq!(pos, Position::new(1, 5));
                        assert_eq!(*dim_name.as_bare_name(), var_name_bare);
                        match dim_name.var_type() {
                            DimType::UserDefined(Positioned { element, .. }) => {
                                assert_eq!(*element, var_type_bare);
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
        assert_parser_err!(
            input,
            ParseError::SyntaxError(
                "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier".to_string()
            )
        );
    }

    #[test]
    fn test_parse_dim_extended_with_qualified_name() {
        let input = "DIM A$ AS STRING";
        assert_parser_err!(input, ParseError::syntax_error("No separator: AS"));
    }

    #[test]
    fn test_parse_dim_user_defined_too_long() {
        let input = "DIM A AS ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNO";
        assert_parser_err!(input, ParseError::IdentifierTooLong);
    }

    #[test]
    fn test_parse_dim_user_defined_cannot_include_period() {
        let input = "DIM A.B AS Card";
        assert_parser_err!(
            input,
            "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING"
        );
    }

    #[test]
    fn test_parse_dim_user_defined_type_cannot_include_period() {
        let input = "DIM Card AS A.B";
        assert_parser_err!(input, ParseError::IdentifierCannotIncludePeriod);
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
                DimNameBuilder::new()
                    .bare_name("A")
                    .dim_type(DimType::Array(
                        vec![ArrayDimension {
                            lbound: None,
                            ubound: 2.as_lit_expr(1, 8)
                        }],
                        Box::new(DimType::BuiltIn(
                            TypeQualifier::DollarString,
                            BuiltInStyle::Compact
                        ))
                    ))
                    .build_list_rc(1, 5)
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
                DimNameBuilder::new()
                    .bare_name("A")
                    .dim_type(DimType::Array(
                        vec![ArrayDimension {
                            lbound: Some(1.as_lit_expr(1, 7)),
                            ubound: 2.as_lit_expr(1, 12)
                        }],
                        Box::new(DimType::Bare)
                    ))
                    .build_list_rc(1, 5)
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
                DimNameBuilder::new()
                    .bare_name("A")
                    .dim_type(DimType::Array(
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
                    ))
                    .build_list_rc(1, 5)
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
                    DimVar::new_compact_local("DIM", TypeQualifier::DollarString)
                        .into_list_rc(1, 5)
                )
            );
        }

        #[test]
        fn test_cannot_assign_to_other_cases_of_keyword() {
            let left_sides = ["DIM", "DIM%", "DIM&", "DIM!", "DIM#"];
            for left_side in &left_sides {
                let input = format!("DIM {}", left_side);
                assert!(matches!(parse_err(&input), ParseError::SyntaxError(_)));
            }
        }
    }
}
