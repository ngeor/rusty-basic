use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc::combine::combine_if_first_some;
use crate::parser::pc::common::*;
use crate::parser::pc::map::{and_then, map};
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use crate::variant;
use std::io::BufRead;

pub fn demand_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    demand(
        expression_node(),
        QError::syntax_error_fn("Expected: expression"),
    )
}

pub fn demand_guarded_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // ws* ( expr )
    // ws+ expr
    demand(
        guarded_expression_node(),
        QError::syntax_error_fn("Expected: expression"),
    )
}

pub fn guarded_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // the order is important because if there is whitespace we can pick up any expression
    // ws+ expr
    // ws* ( expr )
    or(
        guarded_whitespace_expression_node(),
        guarded_parenthesis_expression_node(),
    )
}

fn guarded_parenthesis_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // ws* ( expr )
    map(
        seq3(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(read('('))),
            // caveat: once it reads the opening parenthesis, it goes into demand mode,
            // which is not consistent with the name of the function
            demand_expression_node(),
            demand(read(')'), QError::syntax_error_fn("Expected: )")),
        ),
        |(Locatable { pos, .. }, e, _)| Expression::Parenthesis(Box::new(e)).at(pos),
    )
}

fn guarded_whitespace_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // ws+ expr
    crate::parser::pc::ws::one_or_more_leading(expression_node())
}

pub fn demand_back_guarded_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // ws* ( expr )
    // ws+ expr ws+
    demand(
        or(
            guarded_parenthesis_expression_node(),
            drop_right(seq2(
                guarded_whitespace_expression_node(),
                demand(
                    crate::parser::pc::ws::one_or_more(),
                    QError::syntax_error_fn("Expected: whitespace after expression"),
                ),
            )),
        ),
        QError::syntax_error_fn("Expected: expression"),
    )
}

/// Parses an expression
pub fn expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    map(
        combine_if_first_some(
            // left side
            single_expression_node(),
            // maybe right side
            |first_expr| {
                seq2(
                    operator(first_expr.is_parenthesis()),
                    demand(
                        crate::parser::pc::ws::zero_or_more_leading(lazy(expression_node)),
                        QError::syntax_error_fn("Expected: right side expression"),
                    ),
                )
            },
        ),
        |(left_side, opt_right_side)| {
            (match opt_right_side {
                Some((loc_op, right_side)) => {
                    let Locatable { element: op, pos } = loc_op;
                    left_side.apply_priority_order(right_side, op, pos)
                }
                None => left_side,
            })
            .simplify_unary_minus_literals()
        },
    )
}

fn single_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    or_vec(vec![
        with_pos(string_literal::string_literal()),
        with_pos(word::word()),
        number_literal::number_literal(),
        number_literal::float_without_leading_zero(),
        number_literal::hexadecimal_literal(),
        number_literal::octal_literal(),
        with_pos(parenthesis()),
        unary_not(),
        unary_minus(),
    ])
}

pub fn unary_minus<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    map(
        seq2(
            with_pos(read('-')),
            demand(
                lazy(expression_node),
                QError::syntax_error_fn("Expected: expression after unary minus"),
            ),
        ),
        |(l, r)| r.apply_unary_priority_order(UnaryOperator::Minus, l.pos()),
    )
}

pub fn unary_not<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    map(
        crate::parser::pc::ws::seq2(
            with_pos(keyword(Keyword::Not)),
            demand(
                lazy(expression_node),
                QError::syntax_error_fn("Expected: expression after NOT"),
            ),
            QError::syntax_error_fn("Expected: whitespace after NOT"),
        ),
        |(l, r)| r.apply_unary_priority_order(UnaryOperator::Not, l.pos()),
    )
}

pub fn file_handle<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Locatable<FileHandle>, QError>> {
    and_then(
        seq2(
            with_pos(read('#')),
            demand(
                any_digits(),
                QError::syntax_error_fn("Expected: digits after #"),
            ),
        ),
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

pub fn parenthesis<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Expression, QError>> {
    map(
        in_parenthesis(demand(
            crate::parser::pc::ws::zero_or_more_around(lazy(expression_node)),
            QError::syntax_error_fn("Expected: expression"),
        )),
        |v| Expression::Parenthesis(Box::new(v)),
    )
}

mod string_literal {
    use super::*;

    fn is_not_quote(ch: char) -> bool {
        ch != '"'
    }

    pub fn string_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Expression, QError>> {
        map(
            seq3(
                read('"'),
                crate::parser::pc::str::zero_or_more_if(is_not_quote),
                demand(read('"'), QError::syntax_error_fn("Unterminated string")),
            ),
            |(_, s, _)| Expression::StringLiteral(s),
        )
    }
}

mod number_literal {
    use super::*;
    use crate::variant::BitVec;

    pub fn number_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
        and_then(
            opt_seq3(
                with_pos(any_digits()),
                seq2(
                    with_pos(read('.')),
                    demand(
                        any_digits(),
                        QError::syntax_error_fn("Expected: digits after decimal point"),
                    ),
                ),
                read('#'),
            ),
            |(
                Locatable {
                    element: int_digits,
                    pos,
                },
                opt_frac,
                opt_double,
            )| match opt_frac {
                Some((_, frac_digits)) => {
                    parse_floating_point_literal(int_digits, frac_digits, opt_double.is_some(), pos)
                }
                None => integer_literal_to_expression_node(int_digits, pos),
            },
        )
    }

    pub fn float_without_leading_zero<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
        and_then(
            opt_seq3(
                with_pos(read('.')),
                demand(
                    any_digits(),
                    QError::syntax_error_fn("Expected: digits after decimal point"),
                ),
                read('#'),
            ),
            |(Locatable { pos, .. }, opt_frac_digits, opt_double)| {
                parse_floating_point_literal(
                    "0".to_string(),
                    opt_frac_digits.unwrap(),
                    opt_double.is_some(),
                    pos,
                )
            },
        )
    }

    fn integer_literal_to_expression_node(
        s: String,
        pos: Location,
    ) -> Result<ExpressionNode, QError> {
        match s.parse::<u32>() {
            Ok(u) => {
                if u <= variant::MAX_INTEGER as u32 {
                    Ok(Expression::IntegerLiteral(u as i32).at(pos))
                } else if u <= variant::MAX_LONG as u32 {
                    Ok(Expression::LongLiteral(u as i64).at(pos))
                } else {
                    Ok(Expression::DoubleLiteral(u as f64).at(pos))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn parse_floating_point_literal(
        integer_digits: String,
        fraction_digits: String,
        is_double: bool,
        pos: Location,
    ) -> Result<ExpressionNode, QError> {
        if is_double {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f64>() {
                Ok(f) => Ok(Expression::DoubleLiteral(f).at(pos)),
                Err(err) => Err(err.into()),
            }
        } else {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f32>() {
                Ok(f) => Ok(Expression::SingleLiteral(f).at(pos)),
                Err(err) => Err(err.into()),
            }
        }
    }

    pub fn hexadecimal_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
        hex_or_oct_literal("&H", is_hex_digit, convert_hex_digits)
    }

    pub fn octal_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
        hex_or_oct_literal("&O", is_oct_digit, convert_oct_digits)
    }

    fn hex_or_oct_literal<T: BufRead + 'static>(
        needle: &'static str,
        predicate: fn(char) -> bool,
        converter: fn(String) -> Result<Expression, QError>,
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
        with_pos(and_then(
            and(
                str::str_case_insensitive(needle),
                or(
                    and(read('-'), str::one_or_more_if(predicate)),
                    map(str::one_or_more_if(predicate), |h| ('+', h)),
                ),
            ),
            move |(_ampersand, (sign, digits))| {
                if sign == '-' {
                    Err(QError::Overflow)
                } else {
                    converter(digits)
                }
            },
        ))
    }

    fn is_oct_digit(ch: char) -> bool {
        ch >= '0' && ch <= '7'
    }

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
        if bit_vec.len() == variant::INT_BITS {
            Ok(Expression::IntegerLiteral(bit_vec.into()))
        } else if bit_vec.len() == variant::LONG_BITS {
            Ok(Expression::LongLiteral(bit_vec.into()))
        } else {
            Err(QError::Overflow)
        }
    }
}

mod word {
    use super::*;
    use crate::parser::name::any_word_without_dot;
    use std::convert::TryFrom;

    pub fn word<T: BufRead + 'static>(
    ) -> impl Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Expression, QError> {
        move |r| {
            // read name
            match any_word_without_dot()(r) {
                Ok((r, Some(bare_name_without_dot))) => continue_after_bare_name(
                    r,
                    Expression::VariableName(Name::Bare(bare_name_without_dot)),
                    false,
                ),
                Ok((r, None)) => Ok((r, None)),
                Err(e) => Err(e),
            }
        }
    }

    fn continue_after_bare_name<T: BufRead + 'static>(
        r: EolReader<T>,
        base_expr: Expression,
        has_consecutive_dots: bool,
    ) -> ReaderResult<EolReader<T>, Expression, QError> {
        // we have read so far A or A.B or A..B
        // might encounter: '(', TypeQualifier, or dot
        // if we find '(', we need to fold the existing Expression into a single Name
        match any_symbol()(r) {
            Ok((r, Some(ch))) => {
                if ch == '.' {
                    if has_consecutive_dots {
                        continue_after_bare_name_dot(
                            r,
                            concat_name(base_expr, '.'),
                            has_consecutive_dots,
                        )
                    } else {
                        continue_after_bare_name_dot(r, base_expr, has_consecutive_dots)
                    }
                } else if let Ok(q) = TypeQualifier::try_from(ch) {
                    continue_after_qualified_name(r, qualify(base_expr, q))
                } else if ch == '(' {
                    // fold expr, because it's not possible to invoke or index a property,
                    // so if it has dots, it must be a function (or array name) with dots
                    continue_after_bare_name_parenthesis(r.undo(ch), fold_expr(base_expr))
                } else {
                    // something else, abort
                    Ok((r.undo(ch), Some(base_expr)))
                }
            }
            Ok((r, None)) => Ok((r, Some(base_expr))),
            Err((r, err)) => Err((r, err)),
        }
    }

    fn continue_after_bare_name_dot<T: BufRead + 'static>(
        r: EolReader<T>,
        base_expr: Expression,
        has_consecutive_dots: bool,
    ) -> ReaderResult<EolReader<T>, Expression, QError> {
        // might be a name or a dot or a type qualifier or parenthesis...
        // e.g. A.[B | . | $ | (]
        match any_word_without_dot()(r) {
            Ok((r, Some(bare_name))) => {
                let expr = if has_consecutive_dots {
                    concat_name(base_expr, bare_name)
                } else {
                    Expression::Property(Box::new(base_expr), Name::Bare(bare_name))
                };
                continue_after_bare_name(r, expr, has_consecutive_dots) // recursion
            }
            Ok((r, None)) => {
                // we could not find a name, let's try a symbol
                match any_symbol()(r) {
                    Ok((r, Some(ch))) => {
                        if ch == '.' {
                            if has_consecutive_dots {
                                continue_after_bare_name_dot(
                                    r,
                                    concat_name(base_expr, '.'),
                                    has_consecutive_dots,
                                )
                            } else {
                                // encountered two consecutive periods for the first time,
                                // fold expression into a single name and append two dots
                                if let Name::Bare(folded_name) = fold_name(base_expr) {
                                    continue_after_bare_name_dot(
                                        r,
                                        Expression::VariableName(Name::Bare(
                                            folded_name + '.' + '.',
                                        )),
                                        true,
                                    )
                                } else {
                                    panic!("Expected bare name")
                                }
                            }
                        } else if let Ok(q) = TypeQualifier::try_from(ch) {
                            if has_consecutive_dots {
                                continue_after_qualified_name(r, qualify(base_expr, q))
                            } else {
                                continue_after_qualified_name(
                                    r,
                                    qualify(concat_name(base_expr, '.'), q),
                                )
                            }
                        } else if ch == '(' {
                            continue_after_bare_name_parenthesis(r.undo(ch), fold_expr(base_expr))
                        } else {
                            Ok((r.undo(ch), Some(base_expr)))
                        }
                    }
                    Ok((r, None)) => {
                        // nothing at all, which means we have a trailing dot, which means we need to fold up
                        if has_consecutive_dots {
                            Ok((r, Some(base_expr)))
                        } else {
                            Ok((r, Some(concat_name(fold_expr(base_expr), '.'))))
                        }
                    }
                    Err((r, err)) => Err((r, err)),
                }
            }
            Err((r, err)) => Err((r, err)),
        }
    }

    fn qualify(base_expr: Expression, q: TypeQualifier) -> Expression {
        match base_expr {
            Expression::VariableName(Name::Bare(bare_name)) => {
                Expression::VariableName(Name::Qualified(QualifiedName::new(bare_name, q)))
            }
            Expression::Property(left, Name::Bare(bare_name)) => {
                Expression::Property(left, Name::Qualified(QualifiedName::new(bare_name, q)))
            }
            _ => panic!("Unexpected expression"),
        }
    }

    fn fold_name(base_expr: Expression) -> Name {
        match base_expr {
            Expression::VariableName(name) => name,
            Expression::Property(left, property_name) => {
                let left_name = fold_name(*left);
                match left_name {
                    Name::Bare(bare_left_name) => match property_name {
                        Name::Bare(bare_property_name) => {
                            Name::Bare(bare_left_name + '.' + bare_property_name)
                        }
                        Name::Qualified(QualifiedName {
                            bare_name: bare_property_name,
                            qualifier,
                        }) => Name::Qualified(QualifiedName::new(
                            bare_left_name + '.' + bare_property_name,
                            qualifier,
                        )),
                    },
                    _ => panic!("Left name already qualified, cannot append to it!"),
                }
            }
            _ => panic!("Unexpected expression"),
        }
    }

    fn fold_expr(base_expr: Expression) -> Expression {
        Expression::VariableName(fold_name(base_expr))
    }

    fn concat_name<X>(base_expr: Expression, x: X) -> Expression
    where
        BareName: std::ops::Add<X, Output = BareName>,
    {
        match base_expr {
            Expression::VariableName(Name::Bare(bare_name)) => {
                Expression::VariableName(Name::Bare(bare_name + x))
            }
            _ => panic!("Unsupported expression"),
        }
    }

    fn get_expr_name(base_expr: Expression) -> Name {
        match base_expr {
            Expression::VariableName(name) => name,
            _ => panic!("Unsupported expression"),
        }
    }

    fn continue_after_bare_name_parenthesis<T: BufRead + 'static>(
        r: EolReader<T>,
        base_expr: Expression,
    ) -> ReaderResult<EolReader<T>, Expression, QError> {
        match in_parenthesis(csv_zero_or_more(lazy(expression_node)))(r) {
            Ok((r, Some(args))) => match any_symbol()(r) {
                Ok((r, Some(ch))) => {
                    if ch == '.' {
                        continue_after_bare_name_args_dot(
                            r,
                            Expression::FunctionCall(get_expr_name(base_expr), args),
                        )
                    } else if let Ok(_) = TypeQualifier::try_from(ch) {
                        Err((r, QError::syntax_error("Expected: end of name expression")))
                    } else {
                        Ok((
                            r.undo(ch),
                            Some(Expression::FunctionCall(get_expr_name(base_expr), args)),
                        ))
                    }
                }
                Ok((r, None)) => Ok((
                    r,
                    Some(Expression::FunctionCall(get_expr_name(base_expr), args)),
                )),
                Err((r, err)) => Err((r, err)),
            },
            Ok((r, None)) => {
                // not possible because we peeked '('
                panic!("Should have read parenthesis")
            }
            Err((r, err)) => Err((r, err)),
        }
    }

    fn continue_after_bare_name_args_dot<T: BufRead + 'static>(
        r: EolReader<T>,
        base_expr: Expression,
    ) -> ReaderResult<EolReader<T>, Expression, QError> {
        match any_word_without_dot()(r) {
            Ok((r, Some(bare_property_name))) => continue_after_bare_name_args_dot_bare_name(
                r,
                Expression::Property(Box::new(base_expr), Name::Bare(bare_property_name)),
            ),
            Ok((r, None)) => Err((
                r,
                QError::syntax_error("Expected: property name after period"),
            )),
            Err((r, err)) => Err((r, err)),
        }
    }

    fn continue_after_bare_name_args_dot_bare_name<T: BufRead + 'static>(
        r: EolReader<T>,
        base_expr: Expression,
    ) -> ReaderResult<EolReader<T>, Expression, QError> {
        match any_symbol()(r) {
            Ok((r, Some(ch))) => {
                if ch == '.' {
                    continue_after_bare_name_args_dot(r, base_expr) // recursion
                } else if let Ok(q) = TypeQualifier::try_from(ch) {
                    // e.g. A(1).First$ or A(1).First.Second$
                    let result = qualify(base_expr, q);
                    // pre-emptive strike to see if there is a dot or a type qualifier after this
                    match any_symbol()(r) {
                        Ok((r, Some(ch))) => {
                            if ch == '.' || TypeQualifier::try_from(ch).is_ok() {
                                Err((r, QError::syntax_error("Expected: end of name expr")))
                            } else {
                                Ok((r.undo(ch), Some(result)))
                            }
                        }
                        Ok((r, None)) => Ok((r, Some(result))),
                        Err((r, err)) => Err((r, err)),
                    }
                } else {
                    Ok((r.undo(ch), Some(base_expr)))
                }
            }
            Ok((r, None)) => Ok((r, Some(base_expr))),
            Err((r, err)) => Err((r, err)),
        }
    }

    fn continue_after_qualified_name<T: BufRead + 'static>(
        r: EolReader<T>,
        base_expr: Expression,
    ) -> ReaderResult<EolReader<T>, Expression, QError> {
        // might be A$ followed by '(' (if '.' or type qualifier again it is an error)
        match any_symbol()(r) {
            Ok((r, Some(ch))) => {
                if ch == '(' {
                    // we might be after A$, A.$, A.B$, etc
                    // we need to fold the base_expr as properties cannot have arrays
                    continue_after_qualified_name_parenthesis(r.undo(ch), fold_expr(base_expr))
                } else if let Ok(_) = TypeQualifier::try_from(ch) {
                    Err((r, QError::syntax_error("Expected: end of name expr")))
                } else if ch == '.' {
                    Err((r, QError::syntax_error("Expected: end of name expr")))
                } else {
                    // something else
                    Ok((r.undo(ch), Some(base_expr)))
                }
            }
            Ok((r, None)) => Ok((r, Some(base_expr))),
            Err((r, err)) => Err((r, err)),
        }
    }

    fn continue_after_qualified_name_parenthesis<T: BufRead + 'static>(
        r: EolReader<T>,
        base_expr: Expression,
    ) -> ReaderResult<EolReader<T>, Expression, QError> {
        match in_parenthesis(csv_zero_or_more(lazy(expression_node)))(r) {
            Ok((r, Some(args))) => match any_symbol()(r) {
                Ok((r, Some(ch))) => {
                    if ch == '.' || TypeQualifier::try_from(ch).is_ok() {
                        Err((r, QError::syntax_error("Expected: end of name expression")))
                    } else {
                        Ok((
                            r.undo(ch),
                            Some(Expression::FunctionCall(get_expr_name(base_expr), args)),
                        ))
                    }
                }
                Ok((r, None)) => Ok((
                    r,
                    Some(Expression::FunctionCall(get_expr_name(base_expr), args)),
                )),
                Err((r, err)) => Err((r, err)),
            },
            Ok((r, None)) => {
                // not possible because we peeked '('
                panic!("Should have read parenthesis")
            }
            Err((r, err)) => Err((r, err)),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::parser::test_utils::ExpressionNodeLiteralFactory;

        mod unqualified {
            use super::*;

            mod no_dots {
                use super::*;

                #[test]
                fn test_any_word_without_dot() {
                    let input = "abc";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_array_or_function_no_dot_no_qualifier() {
                    let input = "A(1)";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
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
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_two_consecutive_trailing_dots() {
                    let input = "abc..";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_possible_property() {
                    let input = "a.b.c";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::Property(
                            Box::new(Expression::Property(
                                Box::new(Expression::var("a")),
                                "b".into()
                            )),
                            "c".into()
                        ))
                    );
                }

                #[test]
                fn test_possible_variable() {
                    let input = "a.b.c.";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var("a.b.c.")));
                }

                #[test]
                fn test_bare_array_cannot_have_consecutive_dots_in_properties() {
                    let input = "A(1).O..ops";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, err) = parser(eol_reader).expect_err("Should not parse");
                    assert_eq!(
                        err,
                        QError::syntax_error("Expected: property name after period")
                    );
                }

                #[test]
                fn test_bare_array_bare_property() {
                    let input = "A(1).Suit";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::Property(
                            Box::new(Expression::func("A".into(), vec![1.as_lit_expr(1, 3)])),
                            Name::Bare("Suit".into())
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
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_duplicate_qualifier_is_error() {
                    let input = "abc$%";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, err) = parser(eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }

                #[test]
                fn test_array_or_function_no_dot_qualified() {
                    let input = "A$(1)";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
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
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::Property(
                            Box::new(Expression::var("a".into())),
                            "b$".into()
                        ))
                    );
                }

                #[test]
                fn test_possible_qualified_property_reverts_to_array() {
                    let input = "a.b$(1)";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::func("a.b$".into(), vec![1.as_lit_expr(1, 6)]))
                    );
                }

                #[test]
                fn test_qualified_var_with_dot() {
                    let input = "abc.$";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_qualified_var_with_two_dots() {
                    let input = "abc..$";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(result, Some(Expression::var(input)));
                }

                #[test]
                fn test_dot_after_qualifier_is_error() {
                    let input = "abc$.";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, err) = parser(eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }

                #[test]
                fn test_array_or_function_dotted_qualified() {
                    let input = "A.B$(1)";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::func("A.B$".into(), vec![1.as_lit_expr(1, 6)]))
                    );
                }

                #[test]
                fn test_qualified_array_cannot_have_properties() {
                    let input = "A$(1).Oops";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, err) = parser(eol_reader).expect_err("Should not parse");
                    assert_eq!(
                        err,
                        QError::syntax_error("Expected: end of name expression")
                    );
                }

                #[test]
                fn test_bare_array_qualified_property() {
                    // TODO add linter err test for type mismatch
                    let input = "A(1).Suit$";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, result) = parser(eol_reader).expect("Should parse");
                    assert_eq!(
                        result,
                        Some(Expression::Property(
                            Box::new(Expression::func("A".into(), vec![1.as_lit_expr(1, 3)])),
                            "Suit$".into()
                        ))
                    );
                }

                #[test]
                fn test_bare_array_qualified_property_trailing_dot_is_not_allowed() {
                    let input = "A(1).Suit$.";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, err) = parser(eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }

                #[test]
                fn test_bare_array_qualified_property_extra_qualifier_is_error() {
                    let input = "A(1).Suit$%";
                    let eol_reader = EolReader::from(input);
                    let parser = word();
                    let (_, err) = parser(eol_reader).expect_err("Should not parse");
                    assert_eq!(err, QError::syntax_error("Expected: end of name expr"));
                }
            }
        }
    }
}

pub fn operator<T: BufRead + 'static>(
    had_parenthesis_before: bool,
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Locatable<Operator>, QError>> {
    or_vec(vec![
        crate::parser::pc::ws::zero_or_more_leading(relational_operator()),
        map(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(read('+'))),
            |x| x.map(|_| Operator::Plus),
        ),
        map(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(read('-'))),
            |x| x.map(|_| Operator::Minus),
        ),
        map(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(read('*'))),
            |x| x.map(|_| Operator::Multiply),
        ),
        map(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(read('/'))),
            |x| x.map(|_| Operator::Divide),
        ),
        if had_parenthesis_before {
            // skip whitespace + AND
            map(
                crate::parser::pc::ws::zero_or_more_leading(with_pos(keyword(Keyword::And))),
                |x| x.map(|_| Operator::And),
            )
        } else {
            // demand whitespace + AND
            map(
                crate::parser::pc::ws::one_or_more_leading(with_pos(keyword(Keyword::And))),
                |locatable| locatable.map(|_| Operator::And),
            )
        },
        if had_parenthesis_before {
            // skip whitespace + OR
            map(
                crate::parser::pc::ws::zero_or_more_leading(with_pos(keyword(Keyword::Or))),
                |x| x.map(|_| Operator::Or),
            )
        } else {
            // demand whitespace + OR
            map(
                crate::parser::pc::ws::one_or_more_leading(with_pos(keyword(Keyword::Or))),
                |locatable| locatable.map(|_| Operator::Or),
            )
        },
    ])
}

pub fn lte<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Operator, QError>> {
    and_then(
        opt_seq2(read('<'), read_if(|ch| ch == '=' || ch == '>')),
        |(_, opt_r)| match opt_r {
            Some('=') => Ok(Operator::LessOrEqual),
            Some('>') => Ok(Operator::NotEqual),
            None => Ok(Operator::Less),
            Some(ch) => Err(QError::SyntaxError(format!(
                "Invalid character {} after <",
                ch
            ))),
        },
    )
}

pub fn gte<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Operator, QError>> {
    map(opt_seq2(read('>'), read('=')), |(_, opt_r)| match opt_r {
        Some(_) => Operator::GreaterOrEqual,
        _ => Operator::Greater,
    })
}

pub fn eq<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Operator, QError>> {
    map(read('='), |_| Operator::Equal)
}

pub fn relational_operator<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Locatable<Operator>, QError>> {
    with_pos(or_vec(vec![lte(), gte(), eq()]))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Expression, Operator, Statement, UnaryOperator};
    use crate::{assert_expression, assert_literal_expression, assert_sub_call};

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
                Expression::Property(Box::new(Expression::var("A")), "B".into())
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
                    "Value".into()
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
                            Box::new(1.as_lit_expr(1, 29))
                        )
                        .at_rc(1, 27)]
                    )),
                    "Value".into()
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
                            Box::new("B".as_var_expr(1, 11))
                        )
                        .at_rc(1, 9)
                    ),
                    Box::new("C".as_var_expr(1, 15))
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
                                Box::new("C".as_var_expr(1, 16))
                            )
                            .at_rc(1, 14)
                        ))
                        .at_rc(1, 11)
                    )
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
                            Box::new("C".as_var_expr(1, 15))
                        )
                        .at_rc(1, 13)
                    )
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
                                Box::new("B".as_var_expr(1, 12))
                            )
                            .at_rc(1, 10)
                        ))
                        .at_rc(1, 7)
                    ),
                    Box::new("C".as_var_expr(1, 17)),
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
                        )
                        .at_rc(1, 9)
                    ),
                    Box::new(
                        Expression::BinaryExpression(
                            Operator::Less,
                            Box::new("B".as_var_expr(1, 17)),
                            Box::new(1.as_lit_expr(1, 21)),
                        )
                        .at_rc(1, 19)
                    )
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
                            Box::new(0.as_lit_expr(1, 27))
                        )
                        .at_rc(1, 25)
                    )
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
                    Box::new(2.as_lit_expr(1, 14))
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
                    Box::new(2.as_lit_expr(1, 12))
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
                    Box::new(2.as_lit_expr(1, 12))
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
                            Box::new(2.as_lit_expr(1, 15))
                        )
                        .at_rc(1, 13)
                    )
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
                        )
                        .at_rc(1, 26)],
                    )
                    .at_rc(1, 20)
                ),
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
                Box::new(2.as_lit_expr(1, 13))
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
                            Box::new(2.as_lit_expr(1, 13))
                        )
                        .at_rc(1, 10)
                    ))
                    .at_rc(1, 7)
                ),
                Box::new(3.as_lit_expr(1, 19))
            )
        );
        assert_expression!(
            "1 OR 2",
            Expression::BinaryExpression(
                Operator::Or,
                Box::new(1.as_lit_expr(1, 7)),
                Box::new(2.as_lit_expr(1, 12))
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
                            Box::new(2.as_lit_expr(1, 14))
                        )
                        .at_rc(1, 10)
                    ))
                    .at_rc(1, 7)
                ),
                Box::new(3.as_lit_expr(1, 19))
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
