use std::collections::HashMap;
use std::slice::Iter;

use rusty_common::Positioned;
use rusty_pc::*;

use crate::core::expression::expression_pos_p;
use crate::core::name::bare_name_without_dots;
use crate::core::statement_separator::comments_in_between_keywords;
use crate::error::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::{star_ws, whitespace_ignoring};
use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub struct UserDefinedType {
    /// The name of the type
    name: BareName,
    /// Comments between the type name and the first element
    comments: Vec<Positioned<String>>,
    /// The elements
    elements: Vec<ElementPos>,
}

pub type UserDefinedTypes = HashMap<BareName, UserDefinedType>;

impl UserDefinedType {
    pub fn new(
        name: BareName,
        comments: Vec<Positioned<String>>,
        elements: Vec<ElementPos>,
    ) -> Self {
        Self {
            name,
            comments,
            elements,
        }
    }

    pub fn bare_name(&self) -> &BareName {
        &self.name
    }

    pub fn elements(&self) -> Iter<'_, ElementPos> {
        self.elements.iter()
    }
}

pub type ElementPos = Positioned<Element>;

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    /// The name of the element
    pub name: BareName,
    /// The element type
    pub element_type: ElementType,
    /// Comments between the end of this element and the next one
    pub comments: Vec<Positioned<String>>,
}

impl Element {
    pub fn new(
        name: BareName,
        element_type: ElementType,
        comments: Vec<Positioned<String>>,
    ) -> Self {
        Self {
            name,
            element_type,
            comments,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    Integer,
    Long,
    Single,
    Double,
    FixedLengthString(ExpressionPos, u16),
    UserDefined(BareNamePos),
}

impl ElementType {
    pub fn can_be_referenced_by_property_name(&self, name: &Name) -> bool {
        match name.qualifier() {
            None => true,
            Some(qualifier) => match self {
                Self::Integer => qualifier == TypeQualifier::PercentInteger,
                Self::Long => qualifier == TypeQualifier::AmpersandLong,
                Self::Single => qualifier == TypeQualifier::BangSingle,
                Self::Double => qualifier == TypeQualifier::HashDouble,
                Self::FixedLengthString(_, _) => qualifier == TypeQualifier::DollarString,
                _ => false,
            },
        }
    }
}

impl HasExpressionType for ElementType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::Integer => ExpressionType::BuiltIn(TypeQualifier::PercentInteger),
            Self::Long => ExpressionType::BuiltIn(TypeQualifier::AmpersandLong),
            Self::Single => ExpressionType::BuiltIn(TypeQualifier::BangSingle),
            Self::Double => ExpressionType::BuiltIn(TypeQualifier::HashDouble),
            Self::FixedLengthString(_, l) => ExpressionType::FixedLengthString(*l),
            Self::UserDefined(Positioned { element, .. }) => {
                ExpressionType::UserDefined(element.clone())
            }
        }
    }
}
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

pub fn user_defined_type_p()
-> impl Parser<RcStringView, Output = UserDefinedType, Error = ParseError> {
    seq5(
        keyword_ws_p(Keyword::Type),
        bare_name_without_dots().or_expected("name after TYPE"),
        comments_in_between_keywords(),
        elements_p(),
        keyword_pair(Keyword::End, Keyword::Type),
        |_, name, comments, elements, _| UserDefinedType::new(name, comments, elements),
    )
}

fn elements_p() -> impl Parser<RcStringView, Output = Vec<ElementPos>, Error = ParseError> {
    element_pos_p()
        .one_or_more()
        .or_fail(ParseError::ElementNotDefined)
}

fn element_pos_p() -> impl Parser<RcStringView, Output = ElementPos, Error = ParseError> {
    seq5(
        bare_name_without_dots(),
        whitespace_ignoring(),
        keyword_ws_p(Keyword::As),
        element_type_p().or_expected("element type"),
        comments_in_between_keywords(),
        |element, _, _, element_type, comments| Element::new(element, element_type, comments),
    )
    .with_pos()
}

fn element_type_p() -> impl Parser<RcStringView, Output = ElementType, Error = ParseError> {
    OrParser::new(vec![
        Box::new(keyword_map(&[
            (Keyword::Integer, ElementType::Integer),
            (Keyword::Long, ElementType::Long),
            (Keyword::Single, ElementType::Single),
            (Keyword::Double, ElementType::Double),
        ])),
        Box::new(seq3(
            keyword(Keyword::String),
            star_ws(),
            demand_string_length_p(),
            |_, _, e| ElementType::FixedLengthString(e, 0),
        )),
        Box::new(
            bare_name_without_dots()
                .with_pos()
                .map(ElementType::UserDefined),
        ),
    ])
}

fn demand_string_length_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError>
{
    expression_pos_p().or_expected("string length")
}

#[cfg(test)]
mod tests {
    use rusty_common::AtPos;

    use crate::error::ParseError;
    use crate::test_utils::*;
    use crate::{assert_parser_err, *};

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
            GlobalStatement::UserDefinedType(UserDefinedType::new(
                BareName::from("Card"),
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
            GlobalStatement::UserDefinedType(UserDefinedType::new(
                BareName::from("Card"),
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
        assert_parser_err!(input, ParseError::ElementNotDefined);
    }

    #[test]
    fn type_name_cannot_include_period() {
        let input = "
        TYPE Card.Test
            Suit AS STRING * 9
            Value AS INTEGER
        END TYPE
        ";
        assert_parser_err!(input, ParseError::IdentifierCannotIncludePeriod);
    }

    #[test]
    fn element_name_cannot_include_period() {
        let input = "
        TYPE Card
            Strong.Suit AS STRING * 9
            Value AS INTEGER
        END TYPE
        ";
        assert_parser_err!(input, ParseError::IdentifierCannotIncludePeriod);
    }
}
