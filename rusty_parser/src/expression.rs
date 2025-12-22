use crate::lazy_parser;
use crate::pc::*;
use crate::pc_specific::*;
use crate::types::*;

/// `( expr [, expr]* )`
pub fn in_parenthesis_csv_expressions_non_opt(
    err_msg: &str,
) -> impl Parser<RcStringView, Output = Expressions> + '_ {
    in_parenthesis(csv_expressions_non_opt(err_msg)).no_incomplete()
}

/// Parses one or more expressions separated by comma.
/// FIXME Unlike csv_expressions, the first expression does not need a separator!
pub fn csv_expressions_non_opt(
    msg: &str,
) -> impl Parser<RcStringView, Output = Expressions> + use<'_> {
    csv_non_opt(expression_pos_p(), msg)
}

/// Parses one or more expressions separated by comma.
/// Trailing commas are not allowed.
/// Missing expressions are not allowed.
/// The first expression needs to be preceded by space or surrounded in parenthesis.
pub fn csv_expressions_first_guarded() -> impl Parser<RcStringView, Output = Expressions> {
    AccumulateParser::new(
        ws_expr_pos_p(),
        comma().and_without_undo_keep_right(
            expression_pos_p().or_syntax_error("Expected: expression after comma"),
        ),
    )
}

lazy_parser!(
    pub fn expression_pos_p<I = RcStringView, Output = ExpressionPos> ;
    struct LazyExprParser ;
    eager_expression_pos_p()
);

/// Parses an expression that is either preceded by whitespace
/// or is a parenthesis expression.
///
/// ```text
/// <ws> <expr-not-in-parenthesis> |
/// <ws> <expr-in-parenthesis> |
/// <expr-in-parenthesis>
/// ```
pub fn ws_expr_pos_p() -> impl Parser<RcStringView, Output = ExpressionPos> {
    // ws* ( expr )
    // ws+ expr
    preceded_by_ws(expression_pos_p())
}

/// Parses an expression that is either followed by whitespace
/// or is a parenthesis expression.
///
/// The whitespace is mandatory after a non-parenthesis
/// expression.
///
/// ```text
/// <expr-not-in-parenthesis> <ws> |
/// <expr-in-parenthesis> <ws> |
/// <expr-in-parenthesis>
/// ```
pub fn expr_pos_ws_p() -> impl Parser<RcStringView, Output = ExpressionPos> {
    followed_by_ws(expression_pos_p())
}

/// Parses an expression that is either surrounded by whitespace
/// or is a parenthesis expression.
///
/// The whitespace is mandatory after a non-parenthesis
/// expression.
///
/// ```text
/// <ws> <expr-not-in-parenthesis> <ws> |
/// <ws> <expr-in-parenthesis> <ws> |
/// <expr-in-parenthesis>
/// ```
pub fn ws_expr_pos_ws_p() -> impl Parser<RcStringView, Output = ExpressionPos> {
    followed_by_ws(ws_expr_pos_p())
}

fn preceded_by_ws(
    parser: impl Parser<RcStringView, Output = ExpressionPos>,
) -> impl Parser<RcStringView, Output = ExpressionPos> {
    guard::parser().and_keep_right(parser)
}

fn followed_by_ws(
    parser: impl Parser<RcStringView, Output = ExpressionPos>,
) -> impl Parser<RcStringView, Output = ExpressionPos> {
    parser.chain(
        |expr_pos| {
            let is_paren = expr_pos.is_parenthesis();
            whitespace().allow_none_if(is_paren).no_incomplete()
        },
        |expr_pos, _| expr_pos,
    )
}

/// Parses an expression
fn eager_expression_pos_p() -> impl Parser<RcStringView, Output = ExpressionPos> {
    binary_expression::parser()
}

mod single_or_double_literal {
    use crate::pc::*;
    use crate::pc_specific::{digits, dot, pound, SpecificTrait};
    use crate::*;

    // single ::= <digits> . <digits>
    // single ::= . <digits> (without leading zero)
    // double ::= <digits> . <digits> "#"
    // double ::= . <digits> "#"

    // TODO support more qualifiers besides '#'

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        OptAndPC::new(
            digits(),
            dot().and_without_undo_keep_right(digits().no_incomplete()),
        )
        .and_opt_tuple(pound())
        .flat_map(|input, ((opt_integer_digits, frac_digits), opt_pound)| {
            let left = opt_integer_digits
                .map(|token| token.text)
                .unwrap_or_else(|| "0".to_owned());
            let s = format!("{}.{}", left, frac_digits.text);
            if opt_pound.is_some() {
                match s.parse::<f64>() {
                    Ok(f) => Ok((input, Expression::DoubleLiteral(f))),
                    Err(err) => Err((true, input, err.into())),
                }
            } else {
                match s.parse::<f32>() {
                    Ok(f) => Ok((input, Expression::SingleLiteral(f))),
                    Err(err) => Err((true, input, err.into())),
                }
            }
        })
        .with_pos()
    }
}

mod string_literal {
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::{Expression, ExpressionPos};

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        seq3(
            string_delimiter(),
            inside_string(),
            string_delimiter(),
            |_, token_list, _| Expression::StringLiteral(token_list_to_string(token_list)),
        )
        .with_pos()
    }

    fn string_delimiter() -> impl Parser<RcStringView, Output = Token> {
        any_token_of(TokenType::DoubleQuote)
    }

    fn inside_string() -> impl Parser<RcStringView, Output = TokenList> {
        any_token()
            .filter(|token| {
                !TokenType::DoubleQuote.matches(token) && !TokenType::Eol.matches(token)
            })
            .zero_or_more()
    }
}

mod integer_or_long_literal {
    use crate::pc::*;
    use crate::pc_specific::{SpecificTrait, TokenType};
    use crate::*;
    use rusty_variant::{BitVec, BitVecIntOrLong, MAX_INTEGER, MAX_LONG};

    // result ::= <digits> | <hex-digits> | <oct-digits>
    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        any_token()
            .filter(is_allowed_token)
            .flat_map(process_token)
            .with_pos()
    }

    fn is_allowed_token(token: &Token) -> bool {
        TokenType::Digits.matches(token)
            || TokenType::HexDigits.matches(token)
            || TokenType::OctDigits.matches(token)
    }

    fn process_token(
        input: RcStringView,
        token: Token,
    ) -> ParseResult<RcStringView, Expression, ParseError> {
        let res = match TokenType::from_token(&token) {
            TokenType::Digits => process_dec(token),
            TokenType::HexDigits => process_hex(token),
            TokenType::OctDigits => process_oct(token),
            _ => panic!("Should not have processed {}", token.text),
        };
        match res {
            Ok(expr) => Ok((input, expr)),
            Err(err) => Err((true, input, err)),
        }
    }

    fn process_dec(token: Token) -> Result<Expression, ParseError> {
        match token.text.parse::<u32>() {
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

    fn process_hex(token: Token) -> Result<Expression, ParseError> {
        // token text is &HFFFF or &H-FFFF
        let mut s: String = token.text;
        // remove &
        s.remove(0);
        // remove H
        s.remove(0);
        if s.starts_with('-') {
            Err(ParseError::Overflow)
        } else {
            let mut result: BitVec = BitVec::new();
            for digit in s.chars().skip_while(|ch| *ch == '0') {
                let hex = convert_hex_digit(digit);
                result.push_hex(hex);
            }
            create_expression_from_bit_vec(result)
        }
    }

    fn convert_hex_digit(ch: char) -> u8 {
        if ch.is_ascii_digit() {
            (ch as u8) - b'0'
        } else if ('a'..='f').contains(&ch) {
            (ch as u8) - b'a' + 10
        } else if ('A'..='F').contains(&ch) {
            (ch as u8) - b'A' + 10
        } else {
            panic!("Unexpected hex digit: {}", ch)
        }
    }

    fn process_oct(token: Token) -> Result<Expression, ParseError> {
        let mut s: String = token.text;
        // remove &
        s.remove(0);
        // remove O
        s.remove(0);
        if s.starts_with('-') {
            Err(ParseError::Overflow)
        } else {
            let mut result: BitVec = BitVec::new();
            for digit in s.chars().skip_while(|ch| *ch == '0') {
                let oct = convert_oct_digit(digit);
                result.push_oct(oct);
            }
            create_expression_from_bit_vec(result)
        }
    }

    fn convert_oct_digit(ch: char) -> u8 {
        if ('0'..='7').contains(&ch) {
            (ch as u8) - b'0'
        } else {
            panic!("Unexpected oct digit: {}", ch)
        }
    }

    fn create_expression_from_bit_vec(bit_vec: BitVec) -> Result<Expression, ParseError> {
        bit_vec
            .convert_to_int_or_long_expr()
            .map(|x| match x {
                BitVecIntOrLong::Int(i) => Expression::IntegerLiteral(i),
                BitVecIntOrLong::Long(l) => Expression::LongLiteral(l),
            })
            .map_err(|_| ParseError::Overflow)
    }
}

// TODO consider nesting variable/function_call modules inside property as they are only used there
mod variable {
    use crate::name::{name_with_dots_as_tokens, token_to_type_qualifier, NameAsTokens};
    use crate::pc::*;
    use crate::pc_specific::{SpecificTrait, TokenType};
    use crate::*;
    use std::collections::VecDeque;

    // variable ::= <identifier-with-dots>
    //           |  <identifier-with-dots> <type-qualifier>
    //           |  <keyword> "$"
    //
    // must not be followed by parenthesis (solved by ordering of parsers)
    //
    // if <identifier-with-dots> contains dots, it might be converted to a property expression
    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        name_with_dots_as_tokens().map(map_to_expr).with_pos()
    }

    fn map_to_expr(name_as_tokens: NameAsTokens) -> Expression {
        if is_property_expr(&name_as_tokens) {
            map_to_property(name_as_tokens)
        } else {
            Expression::Variable(name_as_tokens.into(), VariableInfo::unresolved())
        }
    }

    fn is_property_expr(name_as_tokens: &NameAsTokens) -> bool {
        let (names, _) = name_as_tokens;
        let mut name_count = 0;
        let mut last_was_dot = false;
        for name in names {
            if TokenType::Dot.matches(name) {
                if name_count == 0 {
                    panic!("Leading dot cannot happen");
                }
                if last_was_dot {
                    // two dots in a row
                    return false;
                } else {
                    last_was_dot = true;
                }
            } else {
                name_count += 1;
                last_was_dot = false;
            }
        }
        // at least two names and no trailing dot
        name_count > 1 && !last_was_dot
    }

    fn map_to_property(name_as_tokens: NameAsTokens) -> Expression {
        let (names, opt_q_token) = name_as_tokens;
        let mut property_names: VecDeque<String> = names
            .into_iter()
            .filter(|token| !TokenType::Dot.matches(token))
            .map(|token| token.text)
            .collect();
        let mut result = Expression::Variable(
            Name::Bare(BareName::new(property_names.pop_front().unwrap())),
            VariableInfo::unresolved(),
        );
        while let Some(property_name) = property_names.pop_front() {
            let is_last = property_names.is_empty();
            let opt_q_next = if is_last {
                opt_q_token.as_ref().map(token_to_type_qualifier)
            } else {
                None
            };
            result = Expression::Property(
                Box::new(result),
                Name::new(BareName::new(property_name), opt_q_next),
                ExpressionType::Unresolved,
            );
        }
        result
    }
}

mod function_call_or_array_element {
    use crate::expression::expression_pos_p;
    use crate::name::name_with_dots_as_tokens;
    use crate::pc::*;
    use crate::pc_specific::{csv, in_parenthesis, SpecificTrait};
    use crate::*;

    // function_call ::= <function-name> "(" <expr>* ")"
    // function-name ::= <identifier-with-dots>
    //                |  <identifier-with-dots> <type-qualifier>
    //                |  <keyword> "$"
    //
    // Cannot invoke function with empty parenthesis, even if they don't have arguments.
    // However, it is allowed for arrays, so we parse it.
    //
    // A function can be qualified.

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        name_with_dots_as_tokens()
            .and(
                in_parenthesis(csv(expression_pos_p()).or_default()),
                |name_as_tokens, arguments| {
                    Expression::FunctionCall(name_as_tokens.into(), arguments)
                },
            )
            .with_pos()
    }
}

pub mod property {
    use crate::expression::{function_call_or_array_element, variable};
    use crate::name::{identifier, token_to_type_qualifier, type_qualifier};
    use crate::pc::*;
    use crate::pc_specific::{dot, SpecificTrait};
    use crate::*;

    // property ::= <expr> "." <property-name>
    // property-name ::= <identifier-without-dot>
    //                 | <identifier-without-dot> <type-qualifier>
    //
    // expr must not be qualified

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        Seq2::new(base_expr_pos_p(), dot_properties()).flat_map(
            |input, (first_expr_pos, properties)| {
                if !properties.is_empty() && is_qualified(&first_expr_pos.element) {
                    // TODO do this check before parsing the properties
                    return Err((
                        true,
                        input,
                        ParseError::syntax_error("Qualified name cannot have properties"),
                    ));
                }
                let result = properties.into_iter().fold(
                    first_expr_pos,
                    |prev_expr_pos, (name_token, opt_q_token)| {
                        let property_name = Name::new(
                            BareName::from(name_token),
                            opt_q_token.as_ref().map(token_to_type_qualifier),
                        );
                        prev_expr_pos.map(|prev_expr| {
                            Expression::Property(
                                Box::new(prev_expr),
                                property_name,
                                ExpressionType::Unresolved,
                            )
                        })
                    },
                );
                Ok((input, result))
            },
        )
    }

    fn dot_properties() -> impl Parser<RcStringView, Output = Vec<(Token, Option<Token>)>> {
        dot_property().zero_or_more()
    }

    fn dot_property() -> impl Parser<RcStringView, Output = (Token, Option<Token>)> {
        dot().and_without_undo_keep_right(
            property().or_syntax_error("Expected: property name after dot"),
        )
    }

    // cannot be followed by dot or type qualifier if qualified
    fn property() -> impl Parser<RcStringView, Output = (Token, Option<Token>)> {
        identifier().and_opt_tuple(type_qualifier())
    }

    // can't use expression_pos_p because it will stack overflow
    fn base_expr_pos_p() -> impl Parser<RcStringView, Output = ExpressionPos> {
        // order is important, variable matches anything that function_call_or_array_element matches
        OrParser::new(vec![
            Box::new(function_call_or_array_element::parser()),
            Box::new(variable::parser()),
        ])
    }

    fn is_qualified(expr: &Expression) -> bool {
        match expr {
            Expression::Variable(name, _) | Expression::FunctionCall(name, _) => !name.is_bare(),
            _ => {
                panic!("Unexpected property type")
            }
        }
    }
}

mod built_in_function_call {
    use crate::built_ins::built_in_function_call_p;
    use crate::pc::{Parser, RcStringView};
    use crate::pc_specific::SpecificTrait;
    use crate::ExpressionPos;

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        built_in_function_call_p().with_pos()
    }
}

mod binary_expression {
    use crate::expression::{
        built_in_function_call, expression_pos_p, guard, integer_or_long_literal, parenthesis,
        property, single_or_double_literal, string_literal, unary_expression,
    };
    use crate::pc::*;
    use crate::pc_specific::{whitespace, SpecificTrait, TokenType};
    use crate::{
        ExpressionPos, ExpressionPosTrait, ExpressionTrait, Keyword, Operator, ParseError,
    };
    use rusty_common::Positioned;

    // result ::= <non-bin-expr> <operator> <expr>
    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        BinaryExprParser
    }

    struct BinaryExprParser;

    impl Parser<RcStringView> for BinaryExprParser {
        type Output = ExpressionPos;

        fn parse(
            &self,
            tokenizer: RcStringView,
        ) -> ParseResult<RcStringView, Self::Output, ParseError> {
            self.do_parse(tokenizer)
                .map_ok(ExpressionPos::simplify_unary_minus_literals)
        }
    }

    impl BinaryExprParser {
        fn do_parse(
            &self,
            tokenizer: RcStringView,
        ) -> ParseResult<RcStringView, ExpressionPos, ParseError> {
            let (tokenizer, first) = match Self::non_bin_expr().parse(tokenizer) {
                Ok(x) => x,
                Err(err) => return Err(err),
            };

            let is_paren = first.is_parenthesis();
            match Self::operator(is_paren).parse(tokenizer) {
                Ok((
                    tokenizer,
                    Positioned {
                        element: op,
                        pos: op_pos,
                    },
                )) => {
                    let is_keyword_op =
                        op == Operator::And || op == Operator::Or || op == Operator::Modulo;
                    let tokenizer = match guard::parser().parse(tokenizer) {
                        Ok((tokenizer, _)) => tokenizer,
                        Err((false, input, _)) => {
                            if is_keyword_op {
                                return Err((
                                    true,
                                    input,
                                    ParseError::syntax_error("Expected: whitespace or ("),
                                ));
                            } else {
                                input
                            }
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    };
                    expression_pos_p()
                        .or_syntax_error("Expected: expression after operator")
                        .parse(tokenizer)
                        .map_ok(|right| first.apply_priority_order(right, op, op_pos))
                }
                Err((false, tokenizer, _)) => Ok((tokenizer, first)),
                Err(err) => Err(err),
            }
        }

        fn non_bin_expr() -> impl Parser<RcStringView, Output = ExpressionPos> {
            OrParser::new(vec![
                Box::new(single_or_double_literal::parser()),
                Box::new(string_literal::parser()),
                Box::new(integer_or_long_literal::parser()),
                // property internally uses variable and function_call_or_array_element so they can be skipped
                Box::new(property::parser()),
                Box::new(built_in_function_call::parser()),
                Box::new(parenthesis::parser()),
                Box::new(unary_expression::parser()),
            ])
        }

        fn operator(is_paren: bool) -> impl Parser<RcStringView, Output = Positioned<Operator>> {
            OptAndPC::new(
                whitespace(),
                any_token()
                    .filter_map(Self::map_token_to_operator)
                    .with_pos(),
            )
            .flat_map(move |input, (leading_ws, op_pos)| {
                let had_whitespace = leading_ws.is_some();
                let needs_whitespace = matches!(
                    &op_pos.element,
                    Operator::Modulo | Operator::And | Operator::Or
                );
                if had_whitespace || is_paren || !needs_whitespace {
                    Ok((input, op_pos))
                } else {
                    Err((
                        true,
                        input,
                        ParseError::SyntaxError(format!(
                            "Expected: parenthesis before operator {:?}",
                            op_pos.element()
                        )),
                    ))
                }
            })
        }

        fn map_token_to_operator(token: &Token) -> Option<Operator> {
            match TokenType::from_token(token) {
                TokenType::Less => Some(Operator::Less),
                TokenType::LessEquals => Some(Operator::LessOrEqual),
                TokenType::Equals => Some(Operator::Equal),
                TokenType::GreaterEquals => Some(Operator::GreaterOrEqual),
                TokenType::Greater => Some(Operator::Greater),
                TokenType::NotEquals => Some(Operator::NotEqual),
                TokenType::Plus => Some(Operator::Plus),
                TokenType::Minus => Some(Operator::Minus),
                TokenType::Star => Some(Operator::Multiply),
                TokenType::Slash => Some(Operator::Divide),
                TokenType::Keyword => match Keyword::from(token) {
                    Keyword::Mod => Some(Operator::Modulo),
                    Keyword::And => Some(Operator::And),
                    Keyword::Or => Some(Operator::Or),
                    _ => None,
                },
                _ => None,
            }
        }
    }
}

mod unary_expression {
    use crate::expression::{expression_pos_p, guard};
    use crate::pc::*;
    use crate::pc_specific::{keyword, minus_sign, SpecificTrait};
    use crate::{ExpressionPos, ExpressionPosTrait, Keyword, UnaryOperator};
    use rusty_common::Positioned;

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        seq2(
            unary_op(),
            expression_pos_p().or_syntax_error("Expected: expression after unary operator"),
            |Positioned { element: op, pos }, expr| expr.apply_unary_priority_order(op, pos),
        )
    }

    fn unary_op() -> impl Parser<RcStringView, Output = Positioned<UnaryOperator>> {
        minus_sign()
            .map(|_| UnaryOperator::Minus)
            .or(keyword(Keyword::Not)
                .and_without_undo_keep_right(guard::parser().no_incomplete())
                .map(|_| UnaryOperator::Not))
            .with_pos()
    }
}

mod parenthesis {
    use crate::expression::expression_pos_p;
    use crate::pc::*;
    use crate::pc_specific::{in_parenthesis, SpecificTrait};
    use crate::{Expression, ExpressionPos};

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos> {
        in_parenthesis(
            expression_pos_p().or_syntax_error("Expected: expression inside parenthesis"),
        )
        .map(|child| Expression::Parenthesis(Box::new(child)))
        .with_pos()
    }
}

pub mod file_handle {
    //! Used by PRINT and built-ins

    use crate::expression::ws_expr_pos_p;
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::{Expression, ExpressionPos, FileHandle, ParseError};
    use rusty_common::*;

    pub fn file_handle_p() -> impl Parser<RcStringView, Output = Positioned<FileHandle>> {
        // # and digits
        // if # and 0 -> BadFileNameOrNumber
        // if # without digits -> SyntaxError (Expected: digits after #)
        any_token_of(TokenType::Pound)
            .and_tuple(any_token_of(TokenType::Digits).or_syntax_error("Expected: digits after #"))
            .flat_map(|input, (pound, digits)| match digits.text.parse::<u8>() {
                Ok(d) if d > 0 => Ok((input, FileHandle::from(d).at_pos(pound.pos))),
                _ => Err((true, input, ParseError::BadFileNameOrNumber)),
            })
    }

    /// Parses a file handle ( e.g. `#1` ) as an integer literal expression.
    pub fn file_handle_as_expression_pos_p() -> impl Parser<RcStringView, Output = ExpressionPos> {
        file_handle_p().map(|file_handle_pos| file_handle_pos.map(Expression::from))
    }

    pub fn guarded_file_handle_or_expression_p() -> impl Parser<RcStringView, Output = ExpressionPos>
    {
        ws_file_handle().or(ws_expr_pos_p())
    }

    fn ws_file_handle() -> impl Parser<RcStringView, Output = ExpressionPos> {
        whitespace().and_keep_right(file_handle_as_expression_pos_p())
    }
}

pub mod guard {
    use crate::pc::*;
    use crate::pc_specific::{whitespace, TokenType};

    #[derive(Default)]
    pub enum Guard {
        #[default]
        Peeked,
        Whitespace,
    }

    /// `result ::= " " | "("`
    ///
    /// The "(" will be undone.
    pub fn parser() -> impl Parser<RcStringView, Output = Guard> {
        whitespace_guard().or(lparen_guard())
    }

    fn whitespace_guard() -> impl Parser<RcStringView, Output = Guard> {
        whitespace().map(|_| Guard::Whitespace)
    }

    fn lparen_guard() -> impl Parser<RcStringView, Output = Guard> {
        peek_token().flat_map(|input, token| {
            if TokenType::LParen.matches(&token) {
                Ok((input, Guard::Peeked))
            } else {
                default_parse_error(input)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::*;
    use crate::{assert_expression, assert_literal_expression, assert_parser_err, expr};
    use rusty_common::*;

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
        assert_parser_err!(
            "PRINT 1AND 2",
            ParseError::syntax_error("Expected: parenthesis before operator And")
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
        assert_parser_err!(
            "PRINT 1OR 2",
            ParseError::syntax_error("Expected: parenthesis before operator Or")
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

    #[test]
    fn test_whitespace_inside_parenthesis() {
        assert_expression!(
            "( 1 AND 2 )",
            Expression::Parenthesis(Box::new(
                Expression::BinaryExpression(
                    Operator::And,
                    Box::new(1.as_lit_expr(1, 9)),
                    Box::new(2.as_lit_expr(1, 15)),
                    ExpressionType::Unresolved
                )
                .at_rc(1, 11)
            ))
        );
    }

    mod file_handle {
        use super::*;
        use crate::assert_file_handle;

        #[test]
        fn test_valid_file_handles() {
            assert_file_handle!("CLOSE #1", 1);
            assert_file_handle!("CLOSE #2", 2);
            assert_file_handle!("CLOSE #255", 255); // max value
        }

        #[test]
        fn test_file_handle_zero() {
            let input = "CLOSE #0";
            assert_parser_err!(input, ParseError::BadFileNameOrNumber);
        }

        #[test]
        fn test_file_handle_overflow() {
            let input = "CLOSE #256";
            assert_parser_err!(input, ParseError::BadFileNameOrNumber);
        }

        #[test]
        fn test_file_handle_negative() {
            let input = "CLOSE #-1";
            assert_parser_err!(input, ParseError::syntax_error("Expected: digits after #"));
        }
    }

    mod hexadecimal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_parser_err!("PRINT &H-10", ParseError::Overflow);
            assert_parser_err!("PRINT &H100000000", ParseError::Overflow);
        }
    }

    mod octal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_parser_err!("PRINT &O-10", ParseError::Overflow);
            assert_parser_err!("PRINT &O40000000000", ParseError::Overflow);
        }
    }

    mod len {
        use super::*;

        #[test]
        fn len_in_print_must_be_unqualified() {
            let program = r#"PRINT LEN!("hello")"#;
            assert_parser_err!(program, ParseError::syntax_error("Expected: ("), 1, 10);
        }

        #[test]
        fn len_in_assignment_must_be_unqualified() {
            let program = r#"A = LEN!("hello")"#;
            assert_parser_err!(program, ParseError::syntax_error("Expected: ("), 1, 8);
        }
    }

    #[cfg(test)]
    mod name {
        use super::*;

        #[test]
        fn test_var_unresolved() {
            let inputs = ["abc", "abc.", "abc..", "abc$", "abc.$", "abc..$"];
            for input in inputs {
                assert_expression!(var input);
            }
        }

        #[test]
        fn test_func() {
            let inputs = ["A(1)", "A$(1)", "a.b$(1)"];
            for input in inputs {
                assert_expression!(fn input);
            }
        }

        #[test]
        fn test_possible_property() {
            let input = "a.b.c";
            assert_expression!(input, expr!(prop("a", "b", "c")));
        }

        #[test]
        fn test_bare_array_bare_property() {
            let input = "A(1).Suit";
            assert_expression!(
                input,
                expr!(prop(expr!(fn "A", 1.as_lit_expr(1,9)), "Suit"))
            );
        }

        #[test]
        fn test_bare_array_qualified_property() {
            let input = "A(1).Suit$";
            assert_expression!(
                input,
                expr!(prop(expr!(fn "A", 1.as_lit_expr(1,9)), "Suit$"))
            );
        }

        #[test]
        fn test_possible_qualified_property() {
            let input = "a.b$";
            assert_expression!(input, expr!(prop("a"."b$")));
        }

        #[test]
        fn test_bare_array_cannot_have_consecutive_dots_in_properties() {
            let input = "A(1).O..ops";
            assert_parser_err!(input, "Expected: property name after dot");
        }

        #[test]
        fn test_duplicate_qualifier_is_error() {
            let input = "abc$%";
            assert_parser_err!(input, "Identifier cannot end with %, &, !, #, or $");
        }

        #[test]
        fn test_dot_without_properties_is_error() {
            let input = "abc$.";
            assert_parser_err!(input, ParseError::IdentifierCannotIncludePeriod);
        }

        #[test]
        fn test_dot_after_qualifier_is_error() {
            let input = "abc$.hello";
            assert_parser_err!(input, ParseError::IdentifierCannotIncludePeriod);
        }

        #[test]
        fn test_qualified_array_cannot_have_properties() {
            let input = "A$(1).Oops";
            assert_parser_err!(input, "Qualified name cannot have properties");
        }

        #[test]
        fn test_bare_array_qualified_property_trailing_dot_is_not_allowed() {
            let input = "A(1).Suit$.";
            assert_parser_err!(input, ParseError::IdentifierCannotIncludePeriod);
        }

        #[test]
        fn test_bare_array_qualified_property_extra_qualifier_is_error() {
            let input = "A(1).Suit$%";
            assert_parser_err!(input, "Identifier cannot end with %, &, !, #, or $");
        }
    }
}
