use std::marker::PhantomData;

use crate::common::*;
use crate::parser::pc::binary::BinaryParser;
use crate::parser::pc::text::{opt_whitespace_p, whitespace_p, TextParser};
use crate::parser::pc::unary::UnaryParser;
use crate::parser::pc::unary_fn::UnaryFnParser;
use crate::parser::pc::{if_p, is_digit, item_p, Parser, Reader, ReaderResult};
use crate::parser::pc_specific::{in_parenthesis_p, keyword_p, PcSpecific};
use crate::parser::types::*;

fn lazy_expression_node_p<R>() -> LazyExpressionParser<R> {
    LazyExpressionParser(PhantomData)
}

struct LazyExpressionParser<R>(PhantomData<R>);

impl<R> Parser<R> for LazyExpressionParser<R>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    type Output = ExpressionNode;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let parser = expression_node_p();
        parser.parse(reader)
    }
}

pub fn demand_expression_node_p<R>(err_msg: &str) -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression_node_p().or_syntax_error(err_msg)
}

pub fn guarded_expression_node_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    // the order is important because if there is whitespace we can pick up any expression
    // ws+ expr
    // ws* ( expr )
    guarded_whitespace_expression_node_p().or(guarded_parenthesis_expression_node_p())
}

fn guarded_parenthesis_expression_node_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    opt_whitespace_p(false)
        .and(item_p('(').with_pos())
        .and(lazy_expression_node_p())
        .and_demand(
            item_p(')')
                .preceded_by_opt_ws()
                .or_syntax_error("Expected: )"),
        )
        .keep_left()
        .map(|((_, left_parenthesis), child)| {
            Expression::Parenthesis(Box::new(child)).at(left_parenthesis.pos())
        })
}

fn guarded_whitespace_expression_node_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    // ws+ expr
    whitespace_p().and(lazy_expression_node_p()).keep_right()
}

pub fn back_guarded_expression_node_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    // ws* ( expr )
    // ws+ expr ws+
    guarded_parenthesis_expression_node_p().or(guarded_whitespace_expression_node_p()
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after expression"))
        .keep_left())
}

/// Parses an expression
pub fn expression_node_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    single_expression_node_p()
        .and_opt_factory(|first_expr| {
            operator_p(first_expr.is_parenthesis()).and_demand(
                lazy_expression_node_p()
                    .preceded_by_opt_ws()
                    .or_syntax_error("Expected: right side expression"),
            )
        })
        .map(|(left_side, opt_right_side)| {
            (match opt_right_side {
                Some((loc_op, right_side)) => {
                    let Locatable { element: op, pos } = loc_op;
                    left_side.apply_priority_order(right_side, op, pos)
                }
                None => left_side,
            })
            .simplify_unary_minus_literals()
        })
}

fn single_expression_node_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    string_literal::string_literal_p()
        .with_pos()
        .or(word::word_p().with_pos())
        .or(number_literal::number_literal_p())
        .or(number_literal::float_without_leading_zero_p())
        .or(number_literal::hexadecimal_literal_p())
        .or(number_literal::octal_literal_p())
        .or(parenthesis_p().with_pos())
        .or(unary_not_p())
        .or(unary_minus_p())
}

fn unary_minus_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    item_p('-')
        .with_pos()
        .and_demand(
            lazy_expression_node_p().or_syntax_error("Expected: expression after unary minus"),
        )
        .map(|(l, r)| r.apply_unary_priority_order(UnaryOperator::Minus, l.pos()))
}

pub fn unary_not_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Not)
        .with_pos()
        .and_demand(guarded_expression_node_p().or_syntax_error("Expected: expression after NOT"))
        .map(|(l, r)| r.apply_unary_priority_order(UnaryOperator::Not, l.pos()))
}

pub fn file_handle_p<R>() -> impl Parser<R, Output = Locatable<FileHandle>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    item_p('#')
        .with_pos()
        .and_demand(digits_p().or_syntax_error("Expected: digits after #"))
        .and_then(
            |(Locatable { pos, .. }, digits)| match digits.parse::<u8>() {
                Ok(d) => {
                    if d > 0 {
                        Ok(Locatable::new(d.into(), pos))
                    } else {
                        Err(QError::BadFileNameOrNumber)
                    }
                }
                Err(_) => Err(QError::BadFileNameOrNumber),
            },
        )
}

pub fn parenthesis_p<R>() -> impl Parser<R, Output = Expression>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    in_parenthesis_p(
        lazy_expression_node_p().or_syntax_error("Expected: expression inside parenthesis"),
    )
    .map(|child| Expression::Parenthesis(Box::new(child)))
}

mod string_literal {
    use crate::parser::pc::binary::BinaryParser;
    use crate::parser::pc::item_p;
    use crate::parser::pc::unary::UnaryParser;
    use crate::parser::pc::unary_fn::UnaryFnParser;

    use super::*;

    pub fn string_literal_p<R>() -> impl Parser<R, Output = Expression>
    where
        R: Reader<Item = char, Err = QError> + 'static,
    {
        item_p('"')
            .and_opt(non_quote_p())
            .and_demand(item_p('"').or_syntax_error("Unterminated string"))
            .keep_middle()
            .map(|opt_s| Expression::StringLiteral(opt_s.unwrap_or_default()))
    }

    crate::char_sequence_p!(NonQuote, non_quote_p, is_not_quote);

    fn is_not_quote(ch: char) -> bool {
        ch != '"'
    }
}

mod number_literal {
    use super::digits_p;
    use crate::common::*;
    use crate::parser::pc::binary::BinaryParser;
    use crate::parser::pc::text::string_p;
    use crate::parser::pc::unary::UnaryParser;
    use crate::parser::pc::unary_fn::UnaryFnParser;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::PcSpecific;
    use crate::parser::types::*;
    use crate::variant::{BitVec, INT_BITS, LONG_BITS, MAX_INTEGER, MAX_LONG};

    pub fn number_literal_p<R>() -> impl Parser<R, Output = ExpressionNode>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        // TODO support more qualifiers besides '#'
        digits_p()
            .and_opt(
                item_p('.')
                    .and_demand(digits_p().or_syntax_error("Expected: digits after decimal point"))
                    .keep_right(),
            )
            .and_opt(item_p('#'))
            .and_then(
                |((int_digits, opt_fraction_digits), opt_double)| match opt_fraction_digits {
                    Some(fraction_digits) => parse_floating_point_literal_no_pos(
                        int_digits,
                        fraction_digits,
                        opt_double.is_some(),
                    ),
                    _ => integer_literal_to_expression_node_no_pos(int_digits),
                },
            )
            .with_pos()
    }

    pub fn float_without_leading_zero_p<R>() -> impl Parser<R, Output = ExpressionNode>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        item_p('.')
            .and(digits_p())
            .and_opt(item_p('#'))
            .and_then(|((_, fraction_digits), opt_double)| {
                parse_floating_point_literal_no_pos(
                    "0".to_string(),
                    fraction_digits,
                    opt_double.is_some(),
                )
            })
            .with_pos()
    }

    fn integer_literal_to_expression_node_no_pos(s: String) -> Result<Expression, QError> {
        match s.parse::<u32>() {
            Ok(u) => {
                if u <= MAX_INTEGER as u32 {
                    Ok(Expression::IntegerLiteral(u as i32))
                } else if u <= MAX_LONG as u32 {
                    Ok(Expression::LongLiteral(u as i64))
                } else {
                    Ok(Expression::DoubleLiteral(u as f64))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn parse_floating_point_literal_no_pos(
        integer_digits: String,
        fraction_digits: String,
        is_double: bool,
    ) -> Result<Expression, QError> {
        if is_double {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f64>() {
                Ok(f) => Ok(Expression::DoubleLiteral(f)),
                Err(err) => Err(err.into()),
            }
        } else {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f32>() {
                Ok(f) => Ok(Expression::SingleLiteral(f)),
                Err(err) => Err(err.into()),
            }
        }
    }

    macro_rules! hex_or_oct_literal_p {
        ($fn_name:tt, $needle:expr, $recognizer:tt, $converter:tt) => {
            pub fn $fn_name<R>() -> impl Parser<R, Output = ExpressionNode>
            where
                R: Reader<Item = char, Err = QError> + HasLocation + 'static,
            {
                string_p($needle)
                    .and_opt(item_p('-'))
                    .and_demand($recognizer().or_syntax_error("Expected: digits"))
                    .and_then(|((_, opt_negative), digits)| {
                        if opt_negative.is_some() {
                            Err(QError::Overflow)
                        } else {
                            $converter(digits)
                        }
                    })
                    .with_pos()
            }
        };
    }

    hex_or_oct_literal_p!(hexadecimal_literal_p, "&H", hex_digit_p, convert_hex_digits);
    hex_or_oct_literal_p!(octal_literal_p, "&O", oct_digit_p, convert_oct_digits);

    crate::char_sequence_p!(OctDigit, oct_digit_p, is_oct_digit);
    fn is_oct_digit(ch: char) -> bool {
        ch >= '0' && ch <= '7'
    }

    crate::char_sequence_p!(HexDigit, hex_digit_p, is_hex_digit);
    fn is_hex_digit(ch: char) -> bool {
        ch >= '0' && ch <= '9' || ch >= 'a' && ch <= 'f' || ch >= 'A' && ch <= 'F'
    }

    fn convert_hex_digits(digits: String) -> Result<Expression, QError> {
        let mut result: BitVec = BitVec::new();
        for digit in digits.chars().skip_while(|ch| *ch == '0') {
            let hex = convert_hex_digit(digit);
            result.push_hex(hex);
        }
        create_expression_from_bit_vec(result)
    }

    fn convert_hex_digit(ch: char) -> u8 {
        if ch >= '0' && ch <= '9' {
            (ch as u8) - ('0' as u8)
        } else if ch >= 'a' && ch <= 'f' {
            (ch as u8) - ('a' as u8) + 10
        } else if ch >= 'A' && ch <= 'F' {
            (ch as u8) - ('A' as u8) + 10
        } else {
            panic!("Unexpected hex digit: {}", ch)
        }
    }

    fn convert_oct_digits(digits: String) -> Result<Expression, QError> {
        let mut result: BitVec = BitVec::new();
        for digit in digits.chars().skip_while(|ch| *ch == '0') {
            let oct = convert_oct_digit(digit);
            result.push_oct(oct);
        }
        create_expression_from_bit_vec(result)
    }

    fn convert_oct_digit(ch: char) -> u8 {
        if ch >= '0' && ch <= '7' {
            (ch as u8) - ('0' as u8)
        } else {
            panic!("Unexpected oct digit: {}", ch)
        }
    }

    fn create_expression_from_bit_vec(mut bit_vec: BitVec) -> Result<Expression, QError> {
        bit_vec.fit()?;
        if bit_vec.len() == INT_BITS {
            Ok(Expression::IntegerLiteral(bit_vec.into()))
        } else if bit_vec.len() == LONG_BITS {
            Ok(Expression::LongLiteral(bit_vec.into()))
        } else {
            Err(QError::Overflow)
        }
    }
}

pub mod word {
    use std::convert::TryFrom;

    use crate::parser::name::name_with_dot_p;
    use crate::parser::pc::binary::BinaryParser;
    use crate::parser::pc::many::ManyParser;
    use crate::parser::pc::unary::UnaryParser;
    use crate::parser::pc::unary_fn::UnaryFnParser;
    use crate::parser::pc::{any_p, item_p};
    use crate::parser::type_qualifier::type_qualifier_p;

    use super::*;
    use crate::parser::pc_specific::identifier_without_dot_p;

    /*
    //word ::= <name>
    array-prop ::= <name><parens> <dot-property-names>
    name ::= <letter><letter-or-digit-or-dot>*(qualifier)
    parens ::= '(' <expr> , <expr> ')'
    empty-parens ::= '(' <ws>* ')'

    dot-property-names ::= ( '.' <property-name> )*
    property-name ::= <letter><letter-or-digit>*
    */

    pub fn word_p<R>() -> impl Parser<R, Output = Expression>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        name_with_dot_p()
            .and_opt(parenthesis_with_zero_or_more_expressions_p())
            .and_opt(
                dot_property_name()
                    .one_or_more()
                    .and_opt(type_qualifier_p()),
            )
            .and_opt(any_p::<R>().peek_reader_item().validate(|x: &char| {
                if TypeQualifier::try_from(*x).is_ok() || *x == '.' {
                    Err(QError::syntax_error("Expected: end of name expr"))
                } else {
                    Ok(true)
                }
            }))
            .keep_left()
            .and_then(|((name, opt_args), opt_properties)| {
                if opt_args.is_none() && opt_properties.is_none() {
                    // it only makes sense to try to break down a name into properties
                    // when there are no parenthesis and additional properties
                    let (mut name_props, opt_q) = name_to_properties(name.clone());
                    if name_props.len() == 1 {
                        return Ok(Expression::Variable(name, ExpressionType::Unresolved));
                    }
                    let mut base_expr = Expression::Variable(
                        Name::Bare(name_props.remove(0).into()),
                        ExpressionType::Unresolved,
                    );
                    while name_props.len() > 1 {
                        let name_prop = name_props.remove(0);
                        base_expr = Expression::Property(
                            Box::new(base_expr),
                            Name::Bare(name_prop.into()),
                            ExpressionType::Unresolved,
                        );
                    }
                    base_expr = Expression::Property(
                        Box::new(base_expr),
                        Name::new(name_props.remove(0).into(), opt_q),
                        ExpressionType::Unresolved,
                    );
                    return Ok(base_expr);
                }

                if !name.is_bare() && opt_properties.is_some() {
                    return Err(QError::syntax_error(
                        "Qualified name cannot have properties",
                    ));
                }

                let mut base_expr = if let Some(args) = opt_args {
                    Expression::FunctionCall(name, args)
                } else {
                    Expression::Variable(name, ExpressionType::Unresolved)
                };
                if let Some((mut properties, opt_q)) = opt_properties {
                    // take all but last
                    while properties.len() > 1 {
                        base_expr = Expression::Property(
                            Box::new(base_expr),
                            Name::Bare(properties.remove(0).into()),
                            ExpressionType::Unresolved,
                        );
                    }

                    // take last (no need to check for bounds, because it was built with `one_or_more`)
                    base_expr = Expression::Property(
                        Box::new(base_expr),
                        Name::new(properties.remove(0).into(), opt_q),
                        ExpressionType::Unresolved,
                    );

                    Ok(base_expr)
                } else {
                    Ok(base_expr)
                }
            })
    }

    /// Breakdown a name with dots into properties.
    /// If the name has continuous or trailing dots, it is returned as-is.
    fn name_to_properties(name: Name) -> (Vec<String>, Option<TypeQualifier>) {
        let (bare_name, opt_q) = name.into_inner();
        let raw_name: String = bare_name.into();
        let raw_name_copy = raw_name.clone();
        let split = raw_name_copy.split('.');
        let mut parts: Vec<String> = vec![];
        for part in split {
            if part.is_empty() {
                // abort
                parts.clear();
                parts.push(raw_name);
                break;
            } else {
                parts.push(part.to_owned());
            }
        }
        (parts, opt_q)
    }

    fn parenthesis_with_zero_or_more_expressions_p<R>() -> impl Parser<R, Output = ExpressionNodes>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        in_parenthesis_p(lazy_expression_node_p().csv().map_none_to_default())
    }

    fn dot_property_name<R>() -> impl Parser<R, Output = String>
    where
        R: Reader<Item = char, Err = QError> + 'static,
    {
        item_p('.')
            .and_demand(
                identifier_without_dot_p().or_syntax_error("Expected: property name after period"),
            )
            .keep_right()
    }

    #[cfg(test)]
    mod tests {
        use crate::parser::char_reader::EolReader;
        use crate::parser::test_utils::ExpressionNodeLiteralFactory;

        use super::*;

        mod unqualified {
            use super::*;

            mod no_dots {
                use super::*;

                #[test]
                fn test_any_word_without_dot() {
                    let input = "abc";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_array_or_function_no_dot_no_qualifier() {
                    let input = "A(1)";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::func("A".into(), vec![1.as_lit_expr(1, 3)]))
                    );
                }
            }

            mod dots {
                use super::*;

                #[test]
                fn test_trailing_dot() {
                    let input = "abc.";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_two_consecutive_trailing_dots() {
                    let input = "abc..";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_possible_property() {
                    let input = "a.b.c";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::Property(
                            Box::new(Expression::Property(
                                Box::new(Expression::var("a")),
                                "b".into(),
                                ExpressionType::Unresolved
                            )),
                            "c".into(),
                            ExpressionType::Unresolved
                        ))
                    );
                }

                #[test]
                fn test_possible_variable() {
                    let input = "a.b.c.";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var("a.b.c.")));
                }

                #[test]
                fn test_bare_array_cannot_have_consecutive_dots_in_properties() {
                    let input = "A(1).O..ops";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, err) = parser.parse(eol_reader).expect_err("Should not parse");
                    assert_eq!(
                        err,
                        QError::syntax_error("Expected: property name after period")
                    );
                }

                #[test]
                fn test_bare_array_bare_property() {
                    let input = "A(1).Suit";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::Property(
                            Box::new(Expression::func("A".into(), vec![1.as_lit_expr(1, 3)])),
                            Name::Bare("Suit".into()),
                            ExpressionType::Unresolved
                        ))
                    );
                }
            }
        }

        mod qualified {
            use super::*;

            mod no_dots {
                use super::*;

                #[test]
                fn test_qualified_var_without_dot() {
                    let input = "abc$";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_duplicate_qualifier_is_error() {
                    let input = "abc$%";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, err) = parser.parse(eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }

                #[test]
                fn test_array_or_function_no_dot_qualified() {
                    let input = "A$(1)";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::func("A$".into(), vec![1.as_lit_expr(1, 4)]))
                    );
                }
            }

            mod dots {
                use super::*;

                #[test]
                fn test_possible_qualified_property() {
                    let input = "a.b$";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::Property(
                            Box::new(Expression::var("a".into())),
                            "b$".into(),
                            ExpressionType::Unresolved
                        ))
                    );
                }

                #[test]
                fn test_possible_qualified_property_reverts_to_array() {
                    let input = "a.b$(1)";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::func("a.b$".into(), vec![1.as_lit_expr(1, 6)]))
                    );
                }

                #[test]
                fn test_qualified_var_with_dot() {
                    let input = "abc.$";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_qualified_var_with_two_dots() {
                    let input = "abc..$";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_dot_after_qualifier_is_error() {
                    let input = "abc$.";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, err) = parser.parse(eol_reader).expect_err("Should not parse");
                    assert_eq!(
                        err,
                        QError::syntax_error("Expected: property name after period")
                    );
                }

                #[test]
                fn test_array_or_function_dotted_qualified() {
                    let input = "A.B$(1)";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::func("A.B$".into(), vec![1.as_lit_expr(1, 6)]))
                    );
                }

                #[test]
                fn test_qualified_array_cannot_have_properties() {
                    let input = "A$(1).Oops";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, err) = parser.parse(eol_reader).expect_err("Should not parse");
                    assert_eq!(
                        err,
                        QError::syntax_error("Qualified name cannot have properties")
                    );
                }

                #[test]
                fn test_bare_array_qualified_property() {
                    let input = "A(1).Suit$";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, result) = parser.parse(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::Property(
                            Box::new(Expression::func("A".into(), vec![1.as_lit_expr(1, 3)])),
                            "Suit$".into(),
                            ExpressionType::Unresolved
                        ))
                    );
                }

                #[test]
                fn test_bare_array_qualified_property_trailing_dot_is_not_allowed() {
                    let input = "A(1).Suit$.";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, err) = parser.parse(eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }

                #[test]
                fn test_bare_array_qualified_property_extra_qualifier_is_error() {
                    let input = "A(1).Suit$%";
                    let eol_reader = EolReader::from(input);
                    let parser = word_p();
                    let (_, err) = parser.parse(eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }
            }
        }
    }
}

fn operator_p<R>(had_parenthesis_before: bool) -> impl Parser<R, Output = Locatable<Operator>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    relational_operator_p()
        .preceded_by_opt_ws()
        .or(arithmetic_op_p().with_pos().preceded_by_opt_ws())
        .or(and_or_p(
            had_parenthesis_before,
            Keyword::And,
            Operator::And,
        ))
        .or(and_or_p(had_parenthesis_before, Keyword::Or, Operator::Or))
}

fn and_or_p<R>(
    had_parenthesis_before: bool,
    keyword: Keyword,
    operator: Operator,
) -> impl Parser<R, Output = Locatable<Operator>>
where
    R: Reader<Item = char> + HasLocation + 'static,
{
    opt_whitespace_p(!had_parenthesis_before)
        .and(keyword_p(keyword).map(move |_| operator).with_pos())
        .keep_right()
}

fn arithmetic_op_p<R>() -> impl Parser<R, Output = Operator>
where
    R: Reader<Item = char>,
{
    if_p(|ch| ch == '+' || ch == '-' || ch == '*' || ch == '/').map(|ch| match ch {
        '+' => Operator::Plus,
        '-' => Operator::Minus,
        '*' => Operator::Multiply,
        '/' => Operator::Divide,
        _ => panic!("Parser should not have parsed this"),
    })
}

fn lte_p<R>() -> impl Parser<R, Output = Operator>
where
    R: Reader<Item = char> + 'static,
{
    item_p('<')
        .and_opt(if_p(|ch| ch == '=' || ch == '>'))
        .map(|(_, opt_r)| match opt_r {
            Some('=') => Operator::LessOrEqual,
            Some('>') => Operator::NotEqual,
            None => Operator::Less,
            _ => panic!("Parser should not have parsed this"),
        })
}

fn gte_p<R>() -> impl Parser<R, Output = Operator>
where
    R: Reader<Item = char> + 'static,
{
    item_p('>')
        .and_opt(item_p('='))
        .map(|(_, opt_r)| match opt_r {
            Some(_) => Operator::GreaterOrEqual,
            None => Operator::Greater,
        })
}

fn eq_p<R>() -> impl Parser<R, Output = Operator>
where
    R: Reader<Item = char>,
{
    item_p('=').map(|_| Operator::Equal)
}

pub fn relational_operator_p<R>() -> impl Parser<R, Output = Locatable<Operator>>
where
    R: Reader<Item = char> + HasLocation + 'static,
{
    lte_p().or(gte_p()).or(eq_p()).with_pos()
}

// Parses one or more digits.
crate::char_sequence_p!(Digits, digits_p, is_digit);

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::{Expression, ExpressionType, Operator, Statement, UnaryOperator};
    use crate::{assert_expression, assert_literal_expression, assert_sub_call};

    use super::super::test_utils::*;

    #[test]
    fn test_parse_literals() {
        assert_literal_expression!(r#""hello, world""#, "hello, world");
        assert_literal_expression!(r#""hello 123 . AS""#, "hello 123 . AS");
        assert_literal_expression!("42", 42);
        assert_literal_expression!("4.2", 4.2_f32);
        assert_literal_expression!("0.5", 0.5_f32);
        assert_literal_expression!(".5", 0.5_f32);
        assert_literal_expression!("3.14#", 3.14_f64);
        assert_literal_expression!("-42", -42);
    }

    mod variable_expressions {
        use super::*;

        #[test]
        fn test_bare_name() {
            assert_expression!("A", Expression::var("A"));
        }

        #[test]
        fn test_bare_name_with_elements() {
            assert_expression!(
                "A.B",
                Expression::Property(
                    Box::new(Expression::var("A")),
                    "B".into(),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_qualified_name() {
            assert_expression!("A%", Expression::var("A%"));
        }

        #[test]
        fn test_array() {
            assert_expression!("choice$()", Expression::func("choice$", vec![]));
        }

        #[test]
        fn test_array_element_single_dimension() {
            assert_expression!(
                "choice$(1)",
                Expression::func("choice$", vec![1.as_lit_expr(1, 15)])
            );
        }

        #[test]
        fn test_array_element_two_dimensions() {
            assert_expression!(
                "choice$(1, 2)",
                Expression::func("choice$", vec![1.as_lit_expr(1, 15), 2.as_lit_expr(1, 18)])
            );
        }

        #[test]
        fn test_array_element_user_defined_type() {
            assert_expression!(
                "cards(1).Value",
                Expression::Property(
                    Box::new(Expression::func("cards", vec![1.as_lit_expr(1, 13)])),
                    "Value".into(),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_array_element_function_call_as_dimension() {
            assert_expression!(
                "cards(lbound(cards) + 1).Value",
                Expression::Property(
                    Box::new(Expression::func(
                        "cards",
                        vec![Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new(
                                Expression::func("lbound", vec!["cards".as_var_expr(1, 20)])
                                    .at_rc(1, 13)
                            ),
                            Box::new(1.as_lit_expr(1, 29)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 27)]
                    )),
                    "Value".into(),
                    ExpressionType::Unresolved
                )
            );
        }
    }

    mod function_call {
        use super::*;

        #[test]
        fn test_function_call_expression_one_arg() {
            assert_expression!(
                "IsValid(42)",
                Expression::func("IsValid", vec![42.as_lit_expr(1, 15)])
            );
        }

        #[test]
        fn test_function_call_expression_two_args() {
            assert_expression!(
                "CheckProperty(42, \"age\")",
                Expression::func(
                    "CheckProperty",
                    vec![42.as_lit_expr(1, 21), "age".as_lit_expr(1, 25)]
                )
            );
        }

        #[test]
        fn test_function_call_in_function_call() {
            assert_expression!(
                "CheckProperty(LookupName(\"age\"), Confirm(1))",
                Expression::func(
                    "CheckProperty",
                    vec![
                        Expression::func("LookupName", vec!["age".as_lit_expr(1, 32)]).at_rc(1, 21),
                        Expression::func("Confirm", vec![1.as_lit_expr(1, 48)]).at_rc(1, 40)
                    ]
                )
            );
        }
    }

    #[test]
    fn test_lte() {
        assert_expression!(
            "N <= 1",
            Expression::BinaryExpression(
                Operator::LessOrEqual,
                Box::new("N".as_var_expr(1, 7)),
                Box::new(1.as_lit_expr(1, 12)),
                ExpressionType::Unresolved
            )
        );
    }

    #[test]
    fn test_less_than() {
        assert_expression!(
            "A < B",
            Expression::BinaryExpression(
                Operator::Less,
                Box::new("A".as_var_expr(1, 7)),
                Box::new("B".as_var_expr(1, 11)),
                ExpressionType::Unresolved
            )
        );
    }

    mod priority {
        use super::*;

        #[test]
        fn test_a_plus_b_less_than_c() {
            assert_expression!(
                "A + B < C",
                Expression::BinaryExpression(
                    Operator::Less,
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("A".as_var_expr(1, 7)),
                            Box::new("B".as_var_expr(1, 11)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 9)
                    ),
                    Box::new("C".as_var_expr(1, 15)),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_a_plus_parenthesis_b_less_than_c() {
            assert_expression!(
                "A + (B < C)",
                Expression::BinaryExpression(
                    Operator::Plus,
                    Box::new("A".as_var_expr(1, 7)),
                    Box::new(
                        Expression::Parenthesis(Box::new(
                            Expression::BinaryExpression(
                                Operator::Less,
                                Box::new("B".as_var_expr(1, 12)),
                                Box::new("C".as_var_expr(1, 16)),
                                ExpressionType::Unresolved
                            )
                            .at_rc(1, 14)
                        ))
                        .at_rc(1, 11)
                    ),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_a_less_than_b_plus_c() {
            assert_expression!(
                "A < B + C",
                Expression::BinaryExpression(
                    Operator::Less,
                    Box::new("A".as_var_expr(1, 7)),
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("B".as_var_expr(1, 11)),
                            Box::new("C".as_var_expr(1, 15)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 13)
                    ),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_parenthesis_a_less_than_b_end_parenthesis_plus_c() {
            assert_expression!(
                "(A < B) + C",
                Expression::BinaryExpression(
                    Operator::Plus,
                    Box::new(
                        Expression::Parenthesis(Box::new(
                            Expression::BinaryExpression(
                                Operator::Less,
                                Box::new("A".as_var_expr(1, 8)),
                                Box::new("B".as_var_expr(1, 12)),
                                ExpressionType::Unresolved
                            )
                            .at_rc(1, 10)
                        ))
                        .at_rc(1, 7)
                    ),
                    Box::new("C".as_var_expr(1, 17)),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_a_gt_0_and_b_lt_1() {
            assert_expression!(
                "A > 0 AND B < 1",
                Expression::BinaryExpression(
                    Operator::And,
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Greater,
                            Box::new("A".as_var_expr(1, 7)),
                            Box::new(0.as_lit_expr(1, 11)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 9)
                    ),
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Less,
                            Box::new("B".as_var_expr(1, 17)),
                            Box::new(1.as_lit_expr(1, 21)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 19)
                    ),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_not_eof_1_and_id_gt_0() {
            assert_expression!(
                "NOT EOF(1) AND ID > 0",
                Expression::BinaryExpression(
                    Operator::And,
                    Box::new(
                        Expression::UnaryExpression(
                            UnaryOperator::Not,
                            Box::new(
                                Expression::func("EOF", vec![1.as_lit_expr(1, 15)]).at_rc(1, 11)
                            )
                        )
                        .at_rc(1, 7)
                    ),
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Greater,
                            Box::new("ID".as_var_expr(1, 22)),
                            Box::new(0.as_lit_expr(1, 27)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 25)
                    ),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_negated_number_and_positive_number() {
            assert_expression!(
                "-5 AND 2",
                Expression::BinaryExpression(
                    Operator::And,
                    Box::new((-5_i32).as_lit_expr(1, 7)),
                    Box::new(2.as_lit_expr(1, 14)),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_negated_number_plus_positive_number() {
            assert_expression!(
                "-5 + 2",
                Expression::BinaryExpression(
                    Operator::Plus,
                    Box::new((-5_i32).as_lit_expr(1, 7)),
                    Box::new(2.as_lit_expr(1, 12)),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_negated_number_lt_positive_number() {
            assert_expression!(
                "-5 < 2",
                Expression::BinaryExpression(
                    Operator::Less,
                    Box::new((-5_i32).as_lit_expr(1, 7)),
                    Box::new(2.as_lit_expr(1, 12)),
                    ExpressionType::Unresolved
                )
            );
        }
    }

    mod binary_plus {
        use super::*;

        #[test]
        fn test_plus() {
            assert_expression!(
                "N + 1",
                Expression::BinaryExpression(
                    Operator::Plus,
                    Box::new("N".as_var_expr(1, 7)),
                    Box::new(1.as_lit_expr(1, 11)),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_plus_three() {
            assert_expression!(
                "N + 1 + 2",
                Expression::BinaryExpression(
                    Operator::Plus,
                    Box::new("N".as_var_expr(1, 7)),
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new(1.as_lit_expr(1, 11)),
                            Box::new(2.as_lit_expr(1, 15)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 13)
                    ),
                    ExpressionType::Unresolved
                )
            );
        }
    }

    #[test]
    fn test_minus() {
        assert_expression!(
            "N - 2",
            Expression::BinaryExpression(
                Operator::Minus,
                Box::new("N".as_var_expr(1, 7)),
                Box::new(2.as_lit_expr(1, 11)),
                ExpressionType::Unresolved
            )
        );
    }

    #[test]
    fn test_negated_variable() {
        assert_expression!(
            "-N",
            Expression::UnaryExpression(UnaryOperator::Minus, Box::new("N".as_var_expr(1, 8)))
        );
    }

    #[test]
    fn test_negated_number_literal_resolved_eagerly_during_parsing() {
        assert_expression!("-42", Expression::IntegerLiteral(-42));
    }

    #[test]
    fn test_fib_expression() {
        assert_expression!(
            "Fib(N - 1) + Fib(N - 2)",
            Expression::BinaryExpression(
                Operator::Plus,
                Box::new(
                    Expression::func(
                        "Fib",
                        vec![Expression::BinaryExpression(
                            Operator::Minus,
                            Box::new("N".as_var_expr(1, 11)),
                            Box::new(1.as_lit_expr(1, 15)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 13)],
                    )
                    .at_rc(1, 7)
                ),
                Box::new(
                    Expression::func(
                        "Fib",
                        vec![Expression::BinaryExpression(
                            Operator::Minus,
                            Box::new("N".as_var_expr(1, 24)),
                            Box::new(2.as_lit_expr(1, 28)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 26)],
                    )
                    .at_rc(1, 20)
                ),
                ExpressionType::Unresolved
            )
        );
    }

    #[test]
    fn test_negated_function_call() {
        assert_expression!(
            "-Fib(-N)",
            Expression::UnaryExpression(
                UnaryOperator::Minus,
                Box::new(
                    Expression::func(
                        "Fib",
                        vec![Expression::UnaryExpression(
                            UnaryOperator::Minus,
                            Box::new("N".as_var_expr(1, 13)),
                        )
                        .at_rc(1, 12)],
                    )
                    .at_rc(1, 8)
                )
            )
        );
    }

    #[test]
    fn test_and_or_leading_whitespace() {
        assert_expression!(
            "1 AND 2",
            Expression::BinaryExpression(
                Operator::And,
                Box::new(1.as_lit_expr(1, 7)),
                Box::new(2.as_lit_expr(1, 13)),
                ExpressionType::Unresolved
            )
        );
        assert_eq!(
            parse_err("PRINT 1AND 2"),
            QError::syntax_error("No separator: A")
        );
        assert_expression!(
            "(1 OR 2)AND 3",
            Expression::BinaryExpression(
                Operator::And,
                Box::new(
                    Expression::Parenthesis(Box::new(
                        Expression::BinaryExpression(
                            Operator::Or,
                            Box::new(1.as_lit_expr(1, 8)),
                            Box::new(2.as_lit_expr(1, 13)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 10)
                    ))
                    .at_rc(1, 7)
                ),
                Box::new(3.as_lit_expr(1, 19)),
                ExpressionType::Unresolved
            )
        );
        assert_expression!(
            "1 OR 2",
            Expression::BinaryExpression(
                Operator::Or,
                Box::new(1.as_lit_expr(1, 7)),
                Box::new(2.as_lit_expr(1, 12)),
                ExpressionType::Unresolved
            )
        );
        assert_eq!(
            parse_err("PRINT 1OR 2"),
            QError::syntax_error("No separator: O")
        );
        assert_expression!(
            "(1 AND 2)OR 3",
            Expression::BinaryExpression(
                Operator::Or,
                Box::new(
                    Expression::Parenthesis(Box::new(
                        Expression::BinaryExpression(
                            Operator::And,
                            Box::new(1.as_lit_expr(1, 8)),
                            Box::new(2.as_lit_expr(1, 14)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 10)
                    ))
                    .at_rc(1, 7)
                ),
                Box::new(3.as_lit_expr(1, 19)),
                ExpressionType::Unresolved
            )
        );
    }

    mod file_handle {
        use super::*;

        #[test]
        fn test_file_handle_one() {
            let input = "CLOSE #1";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(result, "CLOSE", Expression::IntegerLiteral(1));
        }

        #[test]
        fn test_file_handle_two() {
            let input = "CLOSE #2";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(result, "CLOSE", Expression::IntegerLiteral(2));
        }

        #[test]
        fn test_file_handle_max() {
            let input = "CLOSE #255";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(result, "CLOSE", Expression::IntegerLiteral(255));
        }

        #[test]
        fn test_file_handle_zero() {
            let input = "CLOSE #0";
            assert_eq!(parse_err(input), QError::BadFileNameOrNumber);
        }

        #[test]
        fn test_file_handle_overflow() {
            let input = "CLOSE #256";
            assert_eq!(parse_err(input), QError::BadFileNameOrNumber);
        }

        #[test]
        fn test_file_handle_negative() {
            let input = "CLOSE #-1";
            assert_eq!(
                parse_err(input),
                QError::syntax_error("Expected: digits after #")
            );
        }
    }

    mod hexadecimal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_eq!(parse_err("PRINT &H-10"), QError::Overflow);
            assert_eq!(parse_err("PRINT &H100000000"), QError::Overflow);
        }
    }

    mod octal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_eq!(parse_err("PRINT &O-10"), QError::Overflow);
            assert_eq!(parse_err("PRINT &O40000000000"), QError::Overflow);
        }
    }
}
