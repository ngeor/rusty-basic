// TYPE user-type 'comments
// element-name AS element-type ' comments
// [ element-name AS element-type ]
// END TYPE
// user-type: bare name, cannot include dot despite what the docs say
// Err: Identifier cannot include period
//      Identifier cannot end with %, &, !, #, or $
//      Element not defined (when no elements are defined inside TYPE, or when trying to reference a non existing element)
//      Illegal in procedure or DEF FN (when trying to add this inside a SUB or FUNCTION)
//
// TYPE Card
//   Suit AS STRING * 9 <-- must have * int, CONST is accepted
//                  * >=32768 -> overflow
//                  * >= ~20000 -> out of memory (seems to be before runtime)
//                  *           -> out of string space (seems to be at runtime)
//                  * <= 0 -> Illegal number
//                  * if CONST <= 0 -> throws Out of Memory (seems to be just before runtime)
//   Value AS INTEGER
// END TYPE
// DIM Deck(1 TO 52) AS Card
// Deck(1).Suit = "Club"
// Deck(1).Value = 2
// PRINT Deck(1).Suit, Deck(1).Value

// DIM ace AS Card
// PRINT ace --> Type mismatch

// DIM ace AS Card
// ace.Suit = "Hearts"
// ace.Value = 1
// PRINT ace.Suit, ace.Value --> prints "Hearts <print zone> 1"

// It's possible to assign (copy)
// DIM X AS Address
// DIM Y As Address
// Y = X

// It's not possible to compare (any relational operator)
// It's not possible to use in SELECT CASE
// Passing to SUBs is by ref
// Cannot use the () to pass by value
//
// Type must be defined Before DECLARE SUB

use crate::common::{HasLocation, Locatable, QError};
use crate::parser::comment;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::pc::*;
use crate::parser::pc_specific::{
    demand_keyword_pair_p, keyword_choice_p, keyword_followed_by_whitespace_p, keyword_p,
    PcSpecific,
};
use crate::parser::types::{
    BareName, Element, ElementNode, ElementType, Expression, ExpressionNode, Keyword, Name,
    UserDefinedType,
};

pub fn user_defined_type_p<R>() -> impl Parser<R, Output = UserDefinedType>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_followed_by_whitespace_p(Keyword::Type)
        .and_demand(
            bare_name_without_dot_p()
                .with_pos()
                .or_syntax_error("Expected: name after TYPE"),
        )
        .and_demand(comment::comments_and_whitespace_p())
        .and_demand(element_nodes_p())
        .and_demand(demand_keyword_pair_p(Keyword::End, Keyword::Type))
        .map(|((((_, name), comments), elements), _)| {
            UserDefinedType::new(name, comments, elements)
        })
}

fn bare_name_without_dot_p<R>() -> impl Parser<R, Output = BareName>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    name::name_with_dot_p().and_then(|n| match n {
        Name::Bare(b) => {
            if b.contains('.') {
                Err(QError::IdentifierCannotIncludePeriod)
            } else {
                Ok(b)
            }
        }
        Name::Qualified { .. } => Err(QError::syntax_error(
            "Identifier cannot end with %, &, !, #, or $",
        )),
    })
}

fn element_nodes_p<R>() -> impl Parser<R, Output = Vec<ElementNode>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    element_node_p()
        .one_or_more()
        .or(static_err_p(QError::ElementNotDefined))
}

fn element_node_p<R>() -> impl Parser<R, Output = ElementNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    bare_name_without_dot_p()
        .with_pos()
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after element name"))
        .and_demand(keyword_followed_by_whitespace_p(Keyword::As).or_syntax_error("Expected: AS"))
        .and_demand(element_type_p().or_syntax_error("Expected: element type"))
        .and_demand(comment::comments_and_whitespace_p())
        .map(
            |((((Locatable { element, pos }, _), _), element_type), comments)| {
                Locatable::new(Element::new(element, element_type, comments), pos)
            },
        )
}

fn element_type_p<R>() -> impl Parser<R, Output = ElementType>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_choice_p(&[
        Keyword::Integer,
        Keyword::Long,
        Keyword::Single,
        Keyword::Double,
    ])
    .map(|(k, _)| match k {
        Keyword::Integer => ElementType::Integer,
        Keyword::Long => ElementType::Long,
        Keyword::Single => ElementType::Single,
        Keyword::Double => ElementType::Double,
        _ => panic!("Parser should not have parsed this"),
    })
    .or(keyword_p(Keyword::String_)
        .and_demand(
            item_p('*')
                .surrounded_by_opt_ws()
                .or_syntax_error("Expected: *"),
        )
        .and_demand(demand_string_length_p())
        .keep_right()
        .map(|e| ElementType::FixedLengthString(e, 0)))
    .or(bare_name_without_dot_p()
        .with_pos()
        .map(|n| ElementType::UserDefined(n)))
}

fn demand_string_length_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression::demand_expression_node_p("Expected: string length").and_then(
        |Locatable { element, pos }| match element {
            Expression::IntegerLiteral(i) => {
                if i > 0 && i < crate::variant::MAX_INTEGER {
                    Ok(Locatable::new(element, pos))
                } else {
                    Err(QError::syntax_error("String length out of range"))
                }
            }
            Expression::Variable(_, _) => {
                // allow it, in case it is a CONST
                Ok(Locatable::new(element, pos))
            }
            _ => Err(QError::syntax_error("Illegal string length")),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parser_err;
    use crate::common::AtRowCol;
    use crate::parser::test_utils::*;
    use crate::parser::types::TopLevelToken;

    #[test]
    fn parse_type() {
        let input = "
        TYPE Card
            Suit AS STRING * 9
            Value AS INTEGER
        END TYPE
        ";
        assert_eq!(
            parse(input).demand_single(),
            TopLevelToken::UserDefinedType(UserDefinedType::new(
                BareName::from("Card").at_rc(2, 14),
                vec![],
                vec![
                    Element::new(
                        "Suit".into(),
                        ElementType::FixedLengthString(9.as_lit_expr(3, 30), 0),
                        vec![],
                    )
                    .at_rc(3, 13),
                    Element::new("Value".into(), ElementType::Integer, vec![]).at_rc(4, 13)
                ]
            ))
            .at_rc(2, 9)
        );
    }

    #[test]
    fn parse_comments() {
        let input = "
        TYPE Card ' A card
            Suit AS STRING * 9 ' The suit of the card
            Value AS INTEGER   ' The value of the card
        END TYPE
        ";
        assert_eq!(
            parse(input).demand_single(),
            TopLevelToken::UserDefinedType(UserDefinedType::new(
                BareName::from("Card").at_rc(2, 14),
                vec![String::from(" A card").at_rc(2, 19)],
                vec![
                    Element::new(
                        "Suit".into(),
                        ElementType::FixedLengthString(9.as_lit_expr(3, 30), 0),
                        vec![String::from(" The suit of the card").at_rc(3, 32)],
                    )
                    .at_rc(3, 13),
                    Element::new(
                        "Value".into(),
                        ElementType::Integer,
                        vec![String::from(" The value of the card").at_rc(4, 32)]
                    )
                    .at_rc(4, 13)
                ]
            ))
            .at_rc(2, 9)
        );
    }

    #[test]
    fn no_elements() {
        let input = "
        TYPE Card
        END TYPE
        ";
        assert_parser_err!(input, QError::ElementNotDefined);
    }

    #[test]
    fn string_length_wrong_type() {
        let illegal_expressions = [
            "3.14",
            "6.28#",
            "\"hello\"",
            "(1+1)",
            "Foo(1)",
            "32768", /* MAX_INT (32767) + 1*/
        ];
        for e in &illegal_expressions {
            let input = format!(
                "
            TYPE Invalid
                ZeroString AS STRING * {}
            END TYPE",
                e
            );
            assert_parser_err!(input, QError::syntax_error("Illegal string length"));
        }
    }

    #[test]
    fn string_length_out_of_range() {
        let illegal_expressions = ["0", "-1"];
        for e in &illegal_expressions {
            let input = format!(
                "
            TYPE Invalid
                ZeroString AS STRING * {}
            END TYPE",
                e
            );
            assert_parser_err!(input, QError::syntax_error("String length out of range"));
        }
    }

    #[test]
    fn type_name_cannot_include_period() {
        let input = "
        TYPE Card.Test
            Suit AS STRING * 9
            Value AS INTEGER
        END TYPE
        ";
        assert_parser_err!(input, QError::IdentifierCannotIncludePeriod);
    }

    #[test]
    fn element_name_cannot_include_period() {
        let input = "
        TYPE Card
            Strong.Suit AS STRING * 9
            Value AS INTEGER
        END TYPE
        ";
        assert_parser_err!(input, QError::IdentifierCannotIncludePeriod);
    }
}
