use crate::lazy_parser;
use crate::pc::*;
use crate::pc_specific::*;
use crate::types::*;
use rusty_common::*;

/// `( expr [, expr]* )`
pub fn in_parenthesis_csv_expressions_non_opt(
    err_msg: &str,
) -> impl Parser<Output = ExpressionNodes> + NonOptParser + '_ {
    in_parenthesis(csv_expressions_non_opt(err_msg)).no_incomplete()
}

/// Parses one or more expressions separated by comma.
/// FIXME Unlike csv_expressions, the first expression does not need a separator!
pub fn csv_expressions_non_opt(msg: &str) -> impl Parser<Output = ExpressionNodes> + NonOptParser {
    csv_non_opt(expression_node_p(), msg)
}

/// Parses one or more expressions separated by comma.
/// Trailing commas are not allowed.
/// Missing expressions are not allowed.
/// The first expression needs to be preceded by space or surrounded in parenthesis.
pub fn csv_expressions_first_guarded() -> impl Parser<Output = ExpressionNodes> {
    AccumulateParser::new(
        ws_expr_node(),
        comma()
            .then_demand(expression_node_p().or_syntax_error("Expected: expression after comma")),
    )
}

lazy_parser!(
    pub fn expression_node_p<Output = ExpressionNode> ;
    struct LazyExprParser ;
    eager_expression_node()
);

/// Parses an expression that is either preceded by whitespace
/// or is a parenthesis expression.
///
/// ```text
/// <ws> <expr-not-in-parenthesis> |
/// <ws> <expr-in-parenthesis> |
/// <expr-in-parenthesis>
/// ```
pub fn ws_expr_node() -> impl Parser<Output = ExpressionNode> {
    // ws* ( expr )
    // ws+ expr
    preceded_by_ws(expression_node_p())
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
pub fn expr_node_ws() -> impl Parser<Output = ExpressionNode> {
    followed_by_ws(expression_node_p())
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
pub fn ws_expr_node_ws() -> impl Parser<Output = ExpressionNode> {
    followed_by_ws(ws_expr_node())
}

fn preceded_by_ws(
    parser: impl Parser<Output = ExpressionNode>,
) -> impl Parser<Output = ExpressionNode> {
    guard::parser().and(parser).keep_right()
}

fn followed_by_ws(
    parser: impl Parser<Output = ExpressionNode>,
) -> impl Parser<Output = ExpressionNode> {
    parser.chain(|expr_node| {
        let is_paren = expr_node.is_parenthesis();
        whitespace()
            .allow_none_if(is_paren)
            .no_incomplete()
            .to_parser_once()
            .map(|_| expr_node)
    })
}

/// Parses an expression
fn eager_expression_node() -> impl Parser<Output = ExpressionNode> {
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

    pub fn parser() -> impl Parser<Output = ExpressionNode> {
        OptAndPC::new(digits(), dot().then_demand(digits().no_incomplete()))
            .and_opt(pound())
            .and_then(|((opt_integer_digits, frac_digits), opt_pound)| {
                let left = opt_integer_digits
                    .map(|token| token.text)
                    .unwrap_or_else(|| "0".to_owned());
                let s = format!("{}.{}", left, frac_digits.text);
                if opt_pound.is_some() {
                    match s.parse::<f64>() {
                        Ok(f) => Ok(Expression::DoubleLiteral(f)),
                        Err(err) => Err(err.into()),
                    }
                } else {
                    match s.parse::<f32>() {
                        Ok(f) => Ok(Expression::SingleLiteral(f)),
                        Err(err) => Err(err.into()),
                    }
                }
            })
            .with_pos()
    }
}

mod string_literal {
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::{Expression, ExpressionNode};

    pub fn parser() -> impl Parser<Output = ExpressionNode> {
        seq3(
            string_delimiter(),
            inside_string(),
            string_delimiter().no_incomplete(),
            |_, token_list, _| Expression::StringLiteral(token_list_to_string(token_list)),
        )
        .with_pos()
    }

    fn string_delimiter() -> impl Parser<Output = Token> {
        any_token_of(TokenType::DoubleQuote)
    }

    fn inside_string() -> impl Parser<Output = TokenList> + NonOptParser {
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
    use rusty_common::*;
    use rusty_variant::{BitVec, BitVecIntOrLong, MAX_INTEGER, MAX_LONG};

    // result ::= <digits> | <hex-digits> | <oct-digits>
    pub fn parser() -> impl Parser<Output = ExpressionNode> {
        any_token()
            .filter(is_allowed_token)
            .and_then(process_token)
            .with_pos()
    }

    fn is_allowed_token(token: &Token) -> bool {
        TokenType::Digits.matches(token)
            || TokenType::HexDigits.matches(token)
            || TokenType::OctDigits.matches(token)
    }

    fn process_token(token: Token) -> Result<Expression, QError> {
        match TokenType::from_token(&token) {
            TokenType::Digits => process_dec(token),
            TokenType::HexDigits => process_hex(token),
            TokenType::OctDigits => process_oct(token),
            _ => panic!("Should not have processed {}", token.text),
        }
    }

    fn process_dec(token: Token) -> Result<Expression, QError> {
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

    fn process_hex(token: Token) -> Result<Expression, QError> {
        // token text is &HFFFF or &H-FFFF
        let mut s: String = token.text;
        // remove &
        s.remove(0);
        // remove H
        s.remove(0);
        if s.starts_with('-') {
            Err(QError::Overflow)
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
        if is_digit(ch) {
            (ch as u8) - b'0'
        } else if ('a'..='f').contains(&ch) {
            (ch as u8) - b'a' + 10
        } else if ('A'..='F').contains(&ch) {
            (ch as u8) - b'A' + 10
        } else {
            panic!("Unexpected hex digit: {}", ch)
        }
    }

    fn process_oct(token: Token) -> Result<Expression, QError> {
        let mut s: String = token.text;
        // remove &
        s.remove(0);
        // remove O
        s.remove(0);
        if s.starts_with('-') {
            Err(QError::Overflow)
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

    fn create_expression_from_bit_vec(bit_vec: BitVec) -> Result<Expression, QError> {
        bit_vec.convert_to_int_or_long_expr().map(|x| match x {
            BitVecIntOrLong::Int(i) => Expression::IntegerLiteral(i),
            BitVecIntOrLong::Long(l) => Expression::LongLiteral(l),
        })
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
    pub fn parser() -> impl Parser<Output = ExpressionNode> {
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
    use crate::expression::expression_node_p;
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

    pub fn parser() -> impl Parser<Output = ExpressionNode> {
        name_with_dots_as_tokens()
            .and(in_parenthesis(csv(expression_node_p()).allow_default()))
            .map(|(name_as_tokens, arguments)| {
                Expression::FunctionCall(name_as_tokens.into(), arguments)
            })
            .with_pos()
    }
}

pub mod property {
    use crate::expression::{function_call_or_array_element, variable};
    use crate::name::{identifier, token_to_type_qualifier, type_qualifier};
    use crate::pc::*;
    use crate::pc_specific::{dot, SpecificTrait};
    use crate::*;
    use rusty_common::*;

    // property ::= <expr> "." <property-name>
    // property-name ::= <identifier-without-dot>
    //                 | <identifier-without-dot> <type-qualifier>
    //
    // expr must not be qualified

    pub fn parser() -> impl Parser<Output = ExpressionNode> {
        Seq2::new(base_expr_node(), dot_properties()).and_then(|(first_expr_node, properties)| {
            if !properties.is_empty() && is_qualified(&first_expr_node.element) {
                // TODO do this check before parsing the properties
                return Err(QError::syntax_error(
                    "Qualified name cannot have properties",
                ));
            }
            let result = properties.into_iter().fold(
                first_expr_node,
                |prev_expr_node, (name_token, opt_q_token)| {
                    let property_name = Name::new(
                        BareName::from(name_token),
                        opt_q_token.as_ref().map(token_to_type_qualifier),
                    );
                    prev_expr_node.map(|prev_expr| {
                        Expression::Property(
                            Box::new(prev_expr),
                            property_name,
                            ExpressionType::Unresolved,
                        )
                    })
                },
            );
            Ok(result)
        })
    }

    fn dot_properties() -> impl Parser<Output = Vec<(Token, Option<Token>)>> + NonOptParser {
        dot_property().zero_or_more()
    }

    fn dot_property() -> impl Parser<Output = (Token, Option<Token>)> {
        dot().then_demand(property().or_syntax_error("Expected: property name after dot"))
    }

    // cannot be followed by dot or type qualifier if qualified
    fn property() -> impl Parser<Output = (Token, Option<Token>)> {
        identifier().and_opt(type_qualifier())
    }

    // can't use expression_node_p because it will stack overflow
    fn base_expr_node() -> impl Parser<Output = ExpressionNode> {
        // order is important, variable matches anything that function_call_or_array_element matches
        Alt2::new(function_call_or_array_element::parser(), variable::parser())
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
    use crate::pc::Parser;
    use crate::pc_specific::SpecificTrait;
    use crate::ExpressionNode;

    pub fn parser() -> impl Parser<Output = ExpressionNode> {
        built_in_function_call_p().with_pos()
    }
}

mod binary_expression {
    use crate::expression::{
        built_in_function_call, expression_node_p, guard, integer_or_long_literal, parenthesis,
        property, single_or_double_literal, string_literal, unary_expression,
    };
    use crate::pc::{any_token, Alt7, OptAndPC, Parser, Token, Tokenizer};
    use crate::pc_specific::{whitespace, SpecificTrait, TokenType};
    use crate::{ExpressionNode, ExpressionNodeTrait, ExpressionTrait, Keyword, Operator};
    use rusty_common::{Locatable, ParserErrorTrait, QError};
    use std::str::FromStr;

    // result ::= <non-bin-expr> <operator> <expr>
    pub fn parser() -> impl Parser<Output = ExpressionNode> {
        BinaryExprParser
    }

    struct BinaryExprParser;

    impl Parser for BinaryExprParser {
        type Output = ExpressionNode;

        fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
            self.do_parse(tokenizer)
                .map(ExpressionNode::simplify_unary_minus_literals)
        }
    }

    impl BinaryExprParser {
        fn do_parse(&self, tokenizer: &mut impl Tokenizer) -> Result<ExpressionNode, QError> {
            let first = Self::non_bin_expr().parse(tokenizer)?;
            let is_paren = first.is_parenthesis();
            match Self::operator(is_paren).parse(tokenizer) {
                Ok(Locatable {
                    element: op,
                    pos: op_pos,
                }) => {
                    let is_keyword_op =
                        op == Operator::And || op == Operator::Or || op == Operator::Modulo;
                    if is_keyword_op {
                        guard::parser().no_incomplete().parse(tokenizer)?;
                    } else {
                        guard::parser().allow_none().parse(tokenizer)?;
                    }
                    let right = expression_node_p()
                        .or_syntax_error("Expected: expression after operator")
                        .parse(tokenizer)?;
                    Ok(first.apply_priority_order(right, op, op_pos))
                }
                Err(err) if err.is_incomplete() => Ok(first),
                Err(err) => Err(err),
            }
        }

        fn non_bin_expr() -> impl Parser<Output = ExpressionNode> {
            Alt7::new(
                single_or_double_literal::parser(),
                string_literal::parser(),
                integer_or_long_literal::parser(),
                // property internally uses variable and function_call_or_array_element so they can be skipped
                property::parser(),
                built_in_function_call::parser(),
                parenthesis::parser(),
                unary_expression::parser(),
            )
        }

        fn operator(is_paren: bool) -> impl Parser<Output = Locatable<Operator>> {
            OptAndPC::new(
                whitespace(),
                any_token()
                    .filter_map(Self::map_token_to_operator)
                    .with_pos(),
            )
            .and_then(move |(leading_ws, loc_op)| {
                let had_whitespace = leading_ws.is_some();
                let needs_whitespace = matches!(
                    &loc_op.element,
                    Operator::Modulo | Operator::And | Operator::Or
                );
                if had_whitespace || is_paren || !needs_whitespace {
                    Ok(loc_op)
                } else {
                    Err(QError::SyntaxError(format!(
                        "Expected: parenthesis before operator {:?}",
                        loc_op.element()
                    )))
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
                TokenType::Keyword => {
                    if let Ok(keyword) = Keyword::from_str(&token.text) {
                        match keyword {
                            Keyword::Mod => Some(Operator::Modulo),
                            Keyword::And => Some(Operator::And),
                            Keyword::Or => Some(Operator::Or),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }
}

mod unary_expression {
    use crate::expression::{expression_node_p, guard};
    use crate::pc::{seq2, Parser};
    use crate::pc_specific::{keyword, minus_sign, SpecificTrait};
    use crate::{ExpressionNode, ExpressionNodeTrait, Keyword, UnaryOperator};
    use rusty_common::Locatable;

    pub fn parser() -> impl Parser<Output = ExpressionNode> {
        seq2(
            unary_op(),
            expression_node_p().or_syntax_error("Expected: expression after unary operator"),
            |Locatable { element: op, pos }, expr| expr.apply_unary_priority_order(op, pos),
        )
    }

    fn unary_op() -> impl Parser<Output = Locatable<UnaryOperator>> {
        minus_sign()
            .map(|_| UnaryOperator::Minus)
            .or(keyword(Keyword::Not)
                .then_demand(guard::parser().no_incomplete())
                .map(|_| UnaryOperator::Not))
            .with_pos()
    }
}

mod parenthesis {
    use crate::expression::expression_node_p;
    use crate::pc::Parser;
    use crate::pc_specific::{in_parenthesis, SpecificTrait};
    use crate::{Expression, ExpressionNode};

    pub fn parser() -> impl Parser<Output = ExpressionNode> {
        in_parenthesis(
            expression_node_p().or_syntax_error("Expected: expression inside parenthesis"),
        )
        .map(|child| Expression::Parenthesis(Box::new(child)))
        .with_pos()
    }
}

pub mod file_handle {
    //! Used by PRINT and built-ins

    use crate::expression::ws_expr_node;
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::{Expression, ExpressionNode};
    use rusty_common::*;

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
                Some(token) if TokenType::Pound.matches(&token) => match tokenizer.read()? {
                    Some(token) if TokenType::Digits.matches(&token) => {
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
        file_handle_p().map(|file_handle_node| file_handle_node.map(Expression::from))
    }

    pub fn guarded_file_handle_or_expression_p() -> impl Parser<Output = ExpressionNode> {
        ws_file_handle().or(ws_expr_node())
    }

    fn ws_file_handle() -> impl Parser<Output = ExpressionNode> {
        whitespace()
            .and(file_handle_as_expression_node_p())
            .keep_right()
    }
}

pub mod guard {
    use crate::pc::{Parser, Token, Tokenizer, Undo};
    use crate::pc_specific::{any_token_of, whitespace, TokenType};
    use rusty_common::*;

    pub enum Guard {
        Peeked,
        Whitespace(Token),
    }

    // helps when mapping arguments with `unwrap_or_default`, where the guard
    // is discarded anyway
    impl Default for Guard {
        fn default() -> Self {
            Self::Peeked
        }
    }

    impl Undo for Guard {
        fn undo(self, tokenizer: &mut impl Tokenizer) {
            if let Self::Whitespace(token) = self {
                tokenizer.unread(token);
            }
        }
    }

    /// `result ::= " " | "("`
    ///
    /// The "(" will be undone.
    pub fn parser() -> impl Parser<Output = Guard> {
        whitespace_guard()
            .or(lparen_guard())
            .map_incomplete_err(QError::expected("Expected: whitespace or parenthesis"))
    }

    fn whitespace_guard() -> impl Parser<Output = Guard> {
        whitespace().map(Guard::Whitespace)
    }

    fn lparen_guard() -> impl Parser<Output = Guard> {
        any_token_of(TokenType::LParen)
            .peek()
            .map(|_| Guard::Peeked)
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
            QError::syntax_error("Expected: parenthesis before operator And")
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
            QError::syntax_error("Expected: parenthesis before operator Or")
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
            assert_parser_err!(input, QError::IdentifierCannotIncludePeriod);
        }

        #[test]
        fn test_dot_after_qualifier_is_error() {
            let input = "abc$.hello";
            assert_parser_err!(input, QError::IdentifierCannotIncludePeriod);
        }

        #[test]
        fn test_qualified_array_cannot_have_properties() {
            let input = "A$(1).Oops";
            assert_parser_err!(input, "Qualified name cannot have properties");
        }

        #[test]
        fn test_bare_array_qualified_property_trailing_dot_is_not_allowed() {
            let input = "A(1).Suit$.";
            assert_parser_err!(input, QError::IdentifierCannotIncludePeriod);
        }

        #[test]
        fn test_bare_array_qualified_property_extra_qualifier_is_error() {
            let input = "A(1).Suit$%";
            assert_parser_err!(input, "Identifier cannot end with %, &, !, #, or $");
        }
    }
}
