use crate::built_ins::parser::built_in_function_call_p;
use crate::common::*;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;

/// `( expr [, expr]* )`
// TODO #[deprecated]
pub fn expressions_non_opt(err_msg: &str) -> impl Parser<Output = ExpressionNodes> + '_ {
    in_parenthesis(csv(lazy_expression_node_p(), false).or_syntax_error(err_msg))
        .or_syntax_error("Expected: (")
}

fn parenthesis_with_zero_or_more_expressions_p() -> impl Parser<Output = ExpressionNodes> {
    in_parenthesis(csv(lazy_expression_node_p(), true))
}

pub fn lazy_expression_node_p() -> LazyExpressionParser {
    LazyExpressionParser
}

pub struct LazyExpressionParser;

impl Parser for LazyExpressionParser {
    type Output = ExpressionNode;

    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let parser = expression_node_p();
        parser.parse(reader)
    }
}

// TODO check if all usages are "demand"
// TODO rename to expression_preceded_by_ws
pub fn guarded_expression_node_p() -> impl Parser<Output = ExpressionNode> {
    // ws* ( expr )
    // ws+ expr
    OptAndPC::new(whitespace(), lazy_expression_node_p()).and_then(
        |(opt_leading_whitespace, expression)| {
            let needs_leading_whitespace = !expression.is_parenthesis();
            let has_leading_whitespace = opt_leading_whitespace.is_some();
            if has_leading_whitespace || !needs_leading_whitespace {
                Ok(expression)
            } else {
                Err(QError::syntax_error(
                    "Expected: whitespace before expression",
                ))
            }
        },
    )
}

// TODO rename to expression_surrounded_by_ws
pub fn back_guarded_expression_node_p() -> impl Parser<Output = ExpressionNode> {
    // ws* ( expr ) ws*
    // ws+ expr ws+
    guarded_expression_node_p()
        .and_demand_looking_back(whitespace_boundary_after_expr)
        .keep_left()
}

pub fn expression_node_followed_by_ws() -> impl Parser<Output = ExpressionNode> {
    // ( expr ) ws*
    // expr ws+
    expression_node_p()
        .and_demand_looking_back(whitespace_boundary_after_expr)
        .keep_left()
}

/// Parses an expression
pub fn expression_node_p() -> impl Parser<Output = ExpressionNode> {
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

/// Parses one or more expressions separated by comma.
/// Trailing commas are not allowed.
/// Missing expressions are not allowed.
/// The first expression needs to be preceded by space or surrounded in parenthesis.
pub fn expression_nodes_p() -> impl Parser<Output = ExpressionNodes> {
    // TODO this is a form of ManyParser
    seq2(
        guarded_expression_node_p(),
        comma()
            .then_use(expression_node_p().or_syntax_error("Expected: expression after comma"))
            .zero_or_more(),
        |first, mut remaining| {
            remaining.insert(0, first);
            remaining
        },
    )
}

fn single_expression_node_p() -> impl Parser<Output = ExpressionNode> {
    Alt10::new(
        string_literal::string_literal_p().with_pos(),
        built_in_function_call_p().with_pos(),
        word::word_p().with_pos(),
        number_literal::number_literal_p(),
        number_literal::float_without_leading_zero_p(),
        number_literal::hexadecimal_literal_p().with_pos(),
        number_literal::octal_literal_p().with_pos(),
        parenthesis_p().with_pos(),
        unary_not_p(),
        unary_minus_p(),
    )
}

fn unary_minus_p() -> impl Parser<Output = ExpressionNode> {
    seq2(
        minus_sign().with_pos(),
        lazy_expression_node_p().or_syntax_error("Expected: expression after unary minus"),
        |l, r| r.apply_unary_priority_order(UnaryOperator::Minus, l.pos),
    )
}

pub fn unary_not_p() -> impl Parser<Output = ExpressionNode> {
    seq2(
        keyword(Keyword::Not).with_pos(),
        guarded_expression_node_p().or_syntax_error("Expected: expression after NOT"),
        |l, r| r.apply_unary_priority_order(UnaryOperator::Not, l.pos()),
    )
}

// TODO move the file handle logic into the built_ins as it is only used there

pub fn file_handle_p() -> impl Parser<Output = Locatable<FileHandle>> {
    FileHandleParser
}

// TODO support simple functions without `&self` for cases like FileHandleParser

struct FileHandleParser;

impl Parser for FileHandleParser {
    type Output = Locatable<FileHandle>;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let pos = tokenizer.position();
        match tokenizer.read()? {
            Some(token) if token.kind == TokenType::Pound as i32 => match tokenizer.read()? {
                Some(token) if token.kind == TokenType::Digits as i32 => {
                    match token.text.parse::<u8>() {
                        Ok(d) if d > 0 => Ok(FileHandle::from(d).at(pos)),
                        _ => Err(QError::BadFileNameOrNumber),
                    }
                }
                _ => Err(QError::syntax_error("Expected: digits after #")),
            },
            Some(token) => {
                tokenizer.unread(token);
                Err(QError::Incomplete)
            }
            _ => Err(QError::Incomplete),
        }
    }
}

/// Parses a file handle ( e.g. `#1` ) as an integer literal expression.
pub fn file_handle_as_expression_node_p() -> impl Parser<Output = ExpressionNode> {
    file_handle_p()
        .map(|Locatable { element, pos }| Expression::IntegerLiteral(element.into()).at(pos))
}

pub fn file_handle_or_expression_p() -> impl Parser<Output = ExpressionNode> {
    file_handle_as_expression_node_p().or(expression_node_p())
}

pub fn parenthesis_p() -> impl Parser<Output = Expression> {
    in_parenthesis(
        lazy_expression_node_p().or_syntax_error("Expected: expression inside parenthesis"),
    )
    .map(|child| Expression::Parenthesis(Box::new(child)))
}

pub fn file_handle_comma_p() -> impl Parser<Output = Locatable<FileHandle>> {
    seq2(file_handle_p(), comma(), |l, _| l)
}

pub fn guarded_file_handle_or_expression_p() -> impl Parser<Output = ExpressionNode> {
    file_handle_as_expression_node_p()
        .preceded_by_req_ws()
        .or(guarded_expression_node_p())
}

mod string_literal {
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::Expression;

    pub fn string_literal_p() -> impl Parser<Output = Expression> {
        seq3(
            string_delimiter(),
            inside_string(),
            string_delimiter(),
            |_, token_list, _| Expression::StringLiteral(token_list_to_string(&token_list)),
        )
    }

    fn string_delimiter() -> impl Parser<Output = Token> {
        any_token_of(TokenType::DoubleQuote)
    }

    fn inside_string() -> impl Parser<Output = TokenList> {
        any_token()
            .filter(|token| {
                token.kind != TokenType::DoubleQuote as i32 && token.kind != TokenType::Eol as i32
            })
            .zero_or_more()
    }
}

mod number_literal {
    use crate::common::*;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::types::*;
    use crate::variant::{BitVec, Variant, MAX_INTEGER, MAX_LONG};

    pub fn number_literal_p() -> impl Parser<Output = ExpressionNode> {
        // TODO support more qualifiers besides '#'
        digits()
            .and_opt(dot().then_use(digits()))
            .and_opt(pound())
            .and_then(
                |((int_digits, opt_fraction_digits), opt_double)| match opt_fraction_digits {
                    Some(fraction_digits) => parse_floating_point_literal_no_pos(
                        int_digits.text,
                        fraction_digits.text,
                        opt_double.is_some(),
                    ),
                    _ => integer_literal_to_expression_node_no_pos(int_digits.text),
                },
            )
            .with_pos()
    }

    pub fn float_without_leading_zero_p() -> impl Parser<Output = ExpressionNode> {
        dot()
            .and_demand(digits())
            .and_opt(pound())
            .and_then(|((_, fraction_digits), opt_double)| {
                parse_floating_point_literal_no_pos(
                    "0".to_string(),
                    fraction_digits.text,
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

    fn convert_hex_digits(digits: String) -> Result<Expression, QError> {
        let mut result: BitVec = BitVec::new();
        for digit in digits.chars().skip_while(|ch| *ch == '0') {
            let hex = convert_hex_digit(digit);
            result.push_hex(hex);
        }
        create_expression_from_bit_vec(result)
    }

    fn convert_hex_digit(ch: char) -> u8 {
        if is_digit(ch) {
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

    fn create_expression_from_bit_vec(bit_vec: BitVec) -> Result<Expression, QError> {
        match bit_vec.convert_to_integer_variant()? {
            Variant::VInteger(i) => Ok(Expression::IntegerLiteral(i)),
            Variant::VLong(l) => Ok(Expression::LongLiteral(l)),
            _ => Err(QError::Overflow),
        }
    }

    pub fn hexadecimal_literal_p() -> impl Parser<Output = Expression> {
        any_token_of(TokenType::HexDigits).and_then(|token| {
            // token text is &HFFFF or &H-FFFF
            let mut s: String = token.text;
            // remove &
            s.remove(0);
            // remove H
            s.remove(0);
            if s.starts_with('-') {
                Err(QError::Overflow)
            } else {
                convert_hex_digits(s)
            }
        })
    }

    pub fn octal_literal_p() -> impl Parser<Output = Expression> {
        any_token_of(TokenType::OctDigits).and_then(|token| {
            let mut s: String = token.text;
            // remove &
            s.remove(0);
            // remove O
            s.remove(0);
            if s.starts_with('-') {
                Err(QError::Overflow)
            } else {
                convert_oct_digits(s)
            }
        })
    }

    pub fn digits() -> impl Parser<Output = Token> {
        any_token_of(TokenType::Digits)
    }
}

pub mod word {
    use crate::common::*;
    use crate::parser::expression::parenthesis_with_zero_or_more_expressions_p;
    use crate::parser::name::name_with_dot_p;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::type_qualifier::type_qualifier_p;
    use crate::parser::types::*;
    use std::convert::TryFrom;

    pub fn word_p() -> impl Parser<Output = Expression> {
        name_with_dot_p()
            .and_opt(parenthesis_with_zero_or_more_expressions_p())
            .and_opt(
                // TODO rewrite this
                dot_property_name().and_opt(type_qualifier_p()),
            )
            .and_opt(EnsureEndOfNameParser)
            .keep_left()
            .and_then(|((name, opt_args), opt_properties)| {
                if opt_args.is_none() && opt_properties.is_none() {
                    // it only makes sense to try to break down a name into properties
                    // when there are no parenthesis and additional properties
                    let (mut name_props, opt_q) = name_to_properties(name.clone());
                    if name_props.len() == 1 {
                        return Ok(Expression::Variable(name, VariableInfo::unresolved()));
                    }
                    let mut base_expr = Expression::Variable(
                        Name::Bare(name_props.remove(0).into()),
                        VariableInfo::unresolved(),
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
                    Expression::Variable(name, VariableInfo::unresolved())
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

    // TODO rewrite this
    fn dot_property_name() -> impl Parser<Output = Vec<String>> {
        dot().then_use(Properties)
    }

    struct Properties;

    impl Parser for Properties {
        type Output = Vec<String>;
        fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
            let mut result: Vec<String> = vec![];
            while let Some(token) = tokenizer.read()? {
                if token.kind == TokenType::Keyword as i32 {
                    result.push(token.text);
                } else if token.kind == TokenType::Identifier as i32 {
                    for item in token.text.split('.') {
                        if item.is_empty() {
                            return Err(QError::syntax_error(
                                "Expected: property name after period",
                            ));
                        }
                        result.push(item.to_owned());
                    }
                } else {
                    tokenizer.unread(token);
                    break;
                }
            }
            Ok(result)
        }
    }

    struct EnsureEndOfNameParser;

    impl Parser for EnsureEndOfNameParser {
        type Output = ();
        fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
            match tokenizer.read()? {
                Some(token) => {
                    let token_type = TokenType::try_from(token.kind)?;
                    tokenizer.unread(token);
                    match token_type {
                        TokenType::Dot
                        | TokenType::ExclamationMark
                        | TokenType::Pound
                        | TokenType::Percent
                        | TokenType::Ampersand
                        | TokenType::DollarSign => {
                            Err(QError::syntax_error("Expected: end of name expr"))
                        }
                        _ => Ok(()),
                    }
                }
                None => Ok(()),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::parser::pc_specific::test_helper::create_string_tokenizer;
        use crate::parser::test_utils::ExpressionNodeLiteralFactory;

        use super::*;

        mod unqualified {
            use super::*;

            mod no_dots {
                use super::*;

                #[test]
                fn test_any_word_without_dot() {
                    let input = "abc";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(result, Expression::var_unresolved(input));
                }

                #[test]
                fn test_array_or_function_no_dot_no_qualifier() {
                    let input = "A(1)";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Expression::func("A".into(), vec![1.as_lit_expr(1, 3)])
                    );
                }
            }

            mod dots {
                use super::*;

                #[test]
                fn test_trailing_dot() {
                    let input = "abc.";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(result, Expression::var_unresolved(input));
                }

                #[test]
                fn test_two_consecutive_trailing_dots() {
                    let input = "abc..";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(result, Expression::var_unresolved(input));
                }

                #[test]
                fn test_possible_property() {
                    let input = "a.b.c";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Expression::Property(
                            Box::new(Expression::Property(
                                Box::new(Expression::var_unresolved("a")),
                                "b".into(),
                                ExpressionType::Unresolved
                            )),
                            "c".into(),
                            ExpressionType::Unresolved
                        )
                    );
                }

                #[test]
                fn test_possible_variable() {
                    let input = "a.b.c.";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(result, Expression::var_unresolved("a.b.c."));
                }

                #[test]
                fn test_bare_array_cannot_have_consecutive_dots_in_properties() {
                    let input = "A(1).O..ops";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let err = parser.parse(&mut eol_reader).expect_err("Should not parse");
                    assert_eq!(
                        err,
                        QError::syntax_error("Expected: property name after period")
                    );
                }

                #[test]
                fn test_bare_array_bare_property() {
                    let input = "A(1).Suit";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Expression::Property(
                            Box::new(Expression::func("A".into(), vec![1.as_lit_expr(1, 3)])),
                            Name::Bare("Suit".into()),
                            ExpressionType::Unresolved
                        )
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
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(result, Expression::var_unresolved(input));
                }

                #[test]
                fn test_duplicate_qualifier_is_error() {
                    let input = "abc$%";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let err = parser.parse(&mut eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }

                #[test]
                fn test_array_or_function_no_dot_qualified() {
                    let input = "A$(1)";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Expression::func("A$".into(), vec![1.as_lit_expr(1, 4)])
                    );
                }
            }

            mod dots {
                use super::*;

                #[test]
                fn test_possible_qualified_property() {
                    let input = "a.b$";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Expression::Property(
                            Box::new(Expression::var_unresolved("a".into())),
                            "b$".into(),
                            ExpressionType::Unresolved
                        )
                    );
                }

                #[test]
                fn test_possible_qualified_property_reverts_to_array() {
                    let input = "a.b$(1)";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Expression::func("a.b$".into(), vec![1.as_lit_expr(1, 6)])
                    );
                }

                #[test]
                fn test_qualified_var_with_dot() {
                    let input = "abc.$";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(result, Expression::var_unresolved(input));
                }

                #[test]
                fn test_qualified_var_with_two_dots() {
                    let input = "abc..$";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(result, Expression::var_unresolved(input));
                }

                #[test]
                fn test_dot_after_qualifier_is_error() {
                    let input = "abc$.";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let err = parser.parse(&mut eol_reader).expect_err("Should not parse");
                    assert_eq!(
                        err,
                        QError::syntax_error("Qualified name cannot have properties")
                    );
                }

                #[test]
                fn test_array_or_function_dotted_qualified() {
                    let input = "A.B$(1)";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Expression::func("A.B$".into(), vec![1.as_lit_expr(1, 6)])
                    );
                }

                #[test]
                fn test_qualified_array_cannot_have_properties() {
                    let input = "A$(1).Oops";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let err = parser.parse(&mut eol_reader).expect_err("Should not parse");
                    assert_eq!(
                        err,
                        QError::syntax_error("Qualified name cannot have properties")
                    );
                }

                #[test]
                fn test_bare_array_qualified_property() {
                    let input = "A(1).Suit$";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let result = parser.parse(&mut eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Expression::Property(
                            Box::new(Expression::func("A".into(), vec![1.as_lit_expr(1, 3)])),
                            "Suit$".into(),
                            ExpressionType::Unresolved
                        )
                    );
                }

                #[test]
                fn test_bare_array_qualified_property_trailing_dot_is_not_allowed() {
                    let input = "A(1).Suit$.";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let err = parser.parse(&mut eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }

                #[test]
                fn test_bare_array_qualified_property_extra_qualifier_is_error() {
                    let input = "A(1).Suit$%";
                    let mut eol_reader = create_string_tokenizer(input);
                    let parser = word_p();
                    let err = parser.parse(&mut eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }
            }
        }
    }
}

fn operator_p(had_parenthesis_before: bool) -> impl Parser<Output = Locatable<Operator>> {
    Alt5::new(
        relational_operator_p().preceded_by_opt_ws(),
        arithmetic_op_p().with_pos().preceded_by_opt_ws(),
        modulo_op_p(had_parenthesis_before),
        and_or_p(had_parenthesis_before, Keyword::And, Operator::And),
        and_or_p(had_parenthesis_before, Keyword::Or, Operator::Or),
    )
}

fn and_or_p(
    had_parenthesis_before: bool,
    k: Keyword,
    operator: Operator,
) -> impl Parser<Output = Locatable<Operator>> {
    keyword(k)
        .with_pos()
        .preceded_by_ws(!had_parenthesis_before)
        .map(move |Locatable { pos, .. }| operator.at(pos))
}

struct ArithmeticMap;

impl TokenTypeMap for ArithmeticMap {
    type Output = Operator;
    fn try_map(&self, token_type: TokenType) -> Option<Self::Output> {
        match token_type {
            TokenType::Plus => Some(Operator::Plus),
            TokenType::Minus => Some(Operator::Minus),
            TokenType::Star => Some(Operator::Multiply),
            TokenType::Slash => Some(Operator::Divide),
            _ => None,
        }
    }
}

fn arithmetic_op_p() -> impl Parser<Output = Operator> {
    ArithmeticMap.parser()
}

fn modulo_op_p(had_parenthesis_before: bool) -> impl Parser<Output = Locatable<Operator>> {
    keyword(Keyword::Mod)
        .preceded_by_ws(!had_parenthesis_before)
        .map(|_| Operator::Modulo)
        .with_pos()
}

struct RelationalMap;

impl TokenTypeMap for RelationalMap {
    type Output = Operator;
    fn try_map(&self, token_type: TokenType) -> Option<Self::Output> {
        match token_type {
            TokenType::LessEquals => Some(Operator::LessOrEqual),
            TokenType::GreaterEquals => Some(Operator::GreaterOrEqual),
            TokenType::NotEquals => Some(Operator::NotEqual),
            TokenType::Less => Some(Operator::Less),
            TokenType::Greater => Some(Operator::Greater),
            TokenType::Equals => Some(Operator::Equal),
            _ => None,
        }
    }
}

pub fn relational_operator_p() -> impl Parser<Output = Locatable<Operator>> {
    RelationalMap.parser().with_pos()
}

// TODO there are more test modules earlier, merge them and/or split expression to more modules
#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_parser_err;
    use crate::common::*;
    use crate::parser::{Expression, ExpressionType, Operator, Statement, UnaryOperator};
    use crate::{assert_expression, assert_literal_expression};

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

    #[test]
    fn test_special_characters() {
        assert_literal_expression!(r#""┘""#, "┘");
    }

    mod variable_expressions {
        use super::*;

        #[test]
        fn test_bare_name() {
            assert_expression!("A", Expression::var_unresolved("A"));
        }

        #[test]
        fn test_bare_name_with_elements() {
            assert_expression!(
                "A.B",
                Expression::Property(
                    Box::new(Expression::var_unresolved("A")),
                    "B".into(),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_qualified_name() {
            assert_expression!("A%", Expression::var_unresolved("A%"));
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
        fn test_a_plus_b_minus_c() {
            assert_expression!(
                "A + B - C",
                Expression::BinaryExpression(
                    Operator::Minus,
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
        fn test_a_minus_b_plus_c() {
            assert_expression!(
                "A - B + C",
                Expression::BinaryExpression(
                    Operator::Plus,
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Minus,
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

        #[test]
        fn test_and_two_string_comparisons() {
            assert_expression!(
                r#" "DEF" >= "ABC" AND "DEF" < "GHI" "#,
                Expression::BinaryExpression(
                    Operator::And,
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::GreaterOrEqual,
                            Box::new("DEF".as_lit_expr(1, 8)),
                            Box::new("ABC".as_lit_expr(1, 17)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 14)
                    ),
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Less,
                            Box::new("DEF".as_lit_expr(1, 27)),
                            Box::new("GHI".as_lit_expr(1, 35)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 33)
                    ),
                    ExpressionType::Unresolved
                )
            );
        }

        #[test]
        fn test_or_string_comparison_and_two_string_comparisons() {
            assert_expression!(
                r#" "DEF" >= "ABC" AND "DEF" < "GHI" OR "XYZ" = "XYZ" "#,
                Expression::BinaryExpression(
                    Operator::Or,
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::And,
                            Box::new(
                                Expression::BinaryExpression(
                                    Operator::GreaterOrEqual,
                                    Box::new("DEF".as_lit_expr(1, 8)),
                                    Box::new("ABC".as_lit_expr(1, 17)),
                                    ExpressionType::Unresolved
                                )
                                .at_rc(1, 14)
                            ),
                            Box::new(
                                Expression::BinaryExpression(
                                    Operator::Less,
                                    Box::new("DEF".as_lit_expr(1, 27)),
                                    Box::new("GHI".as_lit_expr(1, 35)),
                                    ExpressionType::Unresolved
                                )
                                .at_rc(1, 33)
                            ),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 23)
                    ),
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Equal,
                            Box::new("XYZ".as_lit_expr(1, 44)),
                            Box::new("XYZ".as_lit_expr(1, 52)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 50)
                    ),
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
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("N".as_var_expr(1, 7)),
                            Box::new(1.as_lit_expr(1, 11)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 9)
                    ),
                    Box::new(2.as_lit_expr(1, 15)),
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
        assert_parser_err!("PRINT 1AND 2", QError::syntax_error("No separator: AND"));
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
        assert_parser_err!("PRINT 1OR 2", QError::syntax_error("No separator: OR"));
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

        macro_rules! assert_file_handle {
            ($input:expr, $expected_file_handle:expr) => {
                let result: Statement = parse($input).demand_single_statement();
                match result {
                    Statement::BuiltInSubCall(_, args) => {
                        assert_eq!(args[0], Expression::IntegerLiteral($expected_file_handle));
                    }
                    _ => {
                        panic!("Expected built-in sub call");
                    }
                }
            };
        }

        #[test]
        fn test_valid_file_handles() {
            assert_file_handle!("CLOSE #1", 1);
            assert_file_handle!("CLOSE #2", 2);
            assert_file_handle!("CLOSE #255", 255); // max value
        }

        #[test]
        fn test_file_handle_zero() {
            let input = "CLOSE #0";
            assert_parser_err!(input, QError::BadFileNameOrNumber);
        }

        #[test]
        fn test_file_handle_overflow() {
            let input = "CLOSE #256";
            assert_parser_err!(input, QError::BadFileNameOrNumber);
        }

        #[test]
        fn test_file_handle_negative() {
            let input = "CLOSE #-1";
            assert_parser_err!(input, QError::syntax_error("Expected: digits after #"));
        }
    }

    mod hexadecimal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_parser_err!("PRINT &H-10", QError::Overflow);
            assert_parser_err!("PRINT &H100000000", QError::Overflow);
        }
    }

    mod octal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_parser_err!("PRINT &O-10", QError::Overflow);
            assert_parser_err!("PRINT &O40000000000", QError::Overflow);
        }
    }

    mod len {
        use super::*;

        #[test]
        fn len_in_print_must_be_unqualified() {
            let program = r#"PRINT LEN!("hello")"#;
            assert_parser_err!(program, QError::syntax_error("Expected: ("), 1, 10);
        }

        #[test]
        fn len_in_assignment_must_be_unqualified() {
            let program = r#"A = LEN!("hello")"#;
            assert_parser_err!(program, QError::syntax_error("Expected: ("), 1, 8);
        }
    }
}
