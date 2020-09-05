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

use crate::common::{Locatable, QError};
use crate::parser::char_reader::EolReader;
use crate::parser::comment;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::pc::common::{demand, or_vec, seq3, seq5, zero_or_more};
use crate::parser::pc::map::{and_then, map, source_and_then_some};
use crate::parser::pc::ws;
use crate::parser::pc::{read, ReaderResult};
use crate::parser::pc_specific::{demand_guarded_keyword, demand_keyword, keyword, with_pos};
use crate::parser::types::{
    BareName, Element, ElementNode, ElementType, Expression, ExpressionNode, Keyword, Name,
    UserDefinedType,
};
use std::io::BufRead;

pub fn user_defined_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, UserDefinedType, QError>> {
    map(
        seq5(
            ws::seq2(
                keyword(Keyword::Type),
                demand(
                    with_pos(bare_name_without_dot()),
                    QError::syntax_error_fn("Expected name after TYPE"),
                ),
                QError::syntax_error_fn("Expected: whitespace after TYPE"),
            ),
            comment::comments(),
            and_then(zero_or_more(element_node()), |v| {
                if v.is_empty() {
                    Err(QError::syntax_error("Element not defined"))
                } else {
                    Ok(v)
                }
            }),
            demand_keyword(Keyword::End),
            demand_guarded_keyword(Keyword::Type),
        ),
        |((_, name), comments, elements, _, _)| UserDefinedType {
            name,
            comments,
            elements,
        },
    )
}

fn bare_name_without_dot<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, BareName, QError>> {
    source_and_then_some(name::name(), |r, n| match n {
        Name::Bare(b) => {
            if b.contains('.') {
                Err((r, QError::syntax_error("Identifier cannot include period")))
            } else {
                Ok((r, Some(b)))
            }
        }
        Name::Qualified { .. } => Err((
            r,
            QError::syntax_error("Identifier cannot end with %, &, !, #, or $"),
        )),
    })
}

fn element_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, (ElementNode, Option<()>), QError>> {
    map(
        seq5(
            with_pos(bare_name_without_dot()),
            demand_guarded_keyword(Keyword::As),
            demand(
                ws::one_or_more(),
                QError::syntax_error_fn("Expected: whitespace after AS"),
            ),
            demand(
                element_type(),
                QError::syntax_error_fn("Expected: element type"),
            ),
            comment::comments(),
        ),
        |(Locatable { element, pos }, _, _, element_type, comments)| {
            (
                Locatable::new(
                    Element {
                        name: element,
                        element_type,
                        comments,
                    },
                    pos,
                ),
                Some(()),
            )
        },
    )
}

fn element_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ElementType, QError>> {
    or_vec(vec![
        map(keyword(Keyword::Integer), |_| ElementType::Integer),
        map(keyword(Keyword::Long), |_| ElementType::Long),
        map(keyword(Keyword::Single), |_| ElementType::Single),
        map(keyword(Keyword::Double), |_| ElementType::Double),
        map(
            seq3(
                keyword(Keyword::String_),
                demand(
                    ws::zero_or_more_around(read('*')),
                    QError::syntax_error_fn("Expected: *"),
                ),
                demand_string_length(),
            ),
            |(_, _, e)| ElementType::String(e),
        ),
        map(with_pos(bare_name_without_dot()), |n| {
            ElementType::UserDefined(n)
        }),
    ])
}

fn demand_string_length<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    and_then(
        expression::demand_expression_node(),
        |Locatable { element, pos }| match element {
            Expression::IntegerLiteral(i) => {
                if i > 0 {
                    Ok(Locatable::new(element, pos))
                } else {
                    Err(QError::syntax_error("Illegal number"))
                }
            }
            Expression::VariableName(_) => {
                // allow it, in case it is a CONST
                Ok(Locatable::new(element, pos))
            }
            _ => Err(QError::syntax_error("Illegal number")),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
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
            TopLevelToken::UserDefinedType(UserDefinedType {
                name: BareName::from("Card").at_rc(2, 14),
                comments: vec![],
                elements: vec![
                    Element {
                        name: "Suit".into(),
                        element_type: ElementType::String(9.as_lit_expr(3, 30)),
                        comments: vec![],
                    }
                    .at_rc(3, 13),
                    Element {
                        name: "Value".into(),
                        element_type: ElementType::Integer,
                        comments: vec![]
                    }
                    .at_rc(4, 13)
                ]
            })
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
            TopLevelToken::UserDefinedType(UserDefinedType {
                name: BareName::from("Card").at_rc(2, 14),
                comments: vec![String::from(" A card").at_rc(2, 19)],
                elements: vec![
                    Element {
                        name: "Suit".into(),
                        element_type: ElementType::String(9.as_lit_expr(3, 30)),
                        comments: vec![String::from(" The suit of the card").at_rc(3, 32)],
                    }
                    .at_rc(3, 13),
                    Element {
                        name: "Value".into(),
                        element_type: ElementType::Integer,
                        comments: vec![String::from(" The value of the card").at_rc(4, 32)]
                    }
                    .at_rc(4, 13)
                ]
            })
            .at_rc(2, 9)
        );
    }

    #[test]
    fn no_elements() {
        let input = "
        TYPE Card
        END TYPE
        ";
        assert_eq!(
            parse_err(input),
            QError::syntax_error("Element not defined")
        );
    }

    #[test]
    fn string_length_wrong_type() {
        let illegal_expressions = [
            "0",
            "-1",
            "3.14",
            "6.28#",
            "\"hello\"",
            "(1+1)",
            "Foo(1)",
            "#1",
        ];
        for e in &illegal_expressions {
            let input = format!(
                "
            TYPE Invalid
                ZeroString AS STRING * {}
            END TYPE",
                e
            );
            assert_eq!(parse_err(input), QError::syntax_error("Illegal number"));
        }
    }

    // TODO no duplicate element name
}
