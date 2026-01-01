use rusty_common::*;
use rusty_pc::*;
use rusty_variant::{MIN_INTEGER, MIN_LONG};

use crate::core::opt_second_expression::conditionally_opt_whitespace;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::{
    BuiltInFunction, ExpressionType, FileHandle, HasExpressionType, Name, Operator, ParseError, TypeQualifier, UnaryOperator, VariableInfo
};

// TODO move traits and logic that is linter specific to linter (including CanCastTo from common)

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    Variable(Name, VariableInfo),
    FunctionCall(Name, Expressions),
    // not a parser type, only at linting can we determine
    // if it's a FunctionCall or an ArrayElement
    ArrayElement(
        // the name of the array (unqualified only for user defined types)
        Name,
        // the array indices
        Expressions,
        // the type of the elements (shared refers to the array itself)
        VariableInfo,
    ),
    BuiltInFunctionCall(BuiltInFunction, Expressions),
    BinaryExpression(
        Operator,
        Box<ExpressionPos>,
        Box<ExpressionPos>,
        ExpressionType,
    ),
    UnaryExpression(UnaryOperator, Box<ExpressionPos>),
    Parenthesis(Box<ExpressionPos>),

    /// A property of a user defined type
    ///
    /// The left side is the expression owning the element,
    /// the right side is the element name.
    ///
    /// Examples:
    ///
    /// - A.B (A left, B right)
    /// - A(1).B ( A(1) left, B right)
    /// - A.B.C (A.B left, C right)
    Property(Box<Self>, Name, ExpressionType),
}

pub type ExpressionPos = Positioned<Expression>;
pub type Expressions = Vec<ExpressionPos>;

impl From<f32> for Expression {
    fn from(f: f32) -> Self {
        Self::SingleLiteral(f)
    }
}

impl From<f64> for Expression {
    fn from(f: f64) -> Self {
        Self::DoubleLiteral(f)
    }
}

impl From<String> for Expression {
    fn from(f: String) -> Self {
        Self::StringLiteral(f)
    }
}

impl From<&str> for Expression {
    fn from(f: &str) -> Self {
        f.to_string().into()
    }
}

impl From<i32> for Expression {
    fn from(f: i32) -> Self {
        Self::IntegerLiteral(f)
    }
}

impl From<i64> for Expression {
    fn from(f: i64) -> Self {
        Self::LongLiteral(f)
    }
}

impl From<FileHandle> for Expression {
    fn from(file_handle: FileHandle) -> Self {
        Self::IntegerLiteral(file_handle.into())
    }
}

impl Expression {
    #[cfg(test)]
    pub fn func(s: &str, args: Expressions) -> Self {
        let name: Name = s.into();
        Expression::FunctionCall(name, args)
    }

    fn unary_minus(child_pos: ExpressionPos) -> Self {
        let Positioned {
            element: child,
            pos,
        } = child_pos;
        match child {
            Self::SingleLiteral(n) => Self::SingleLiteral(-n),
            Self::DoubleLiteral(n) => Self::DoubleLiteral(-n),
            Self::IntegerLiteral(n) => {
                if n <= MIN_INTEGER {
                    Self::LongLiteral(-n as i64)
                } else {
                    Self::IntegerLiteral(-n)
                }
            }
            Self::LongLiteral(n) => {
                if n <= MIN_LONG {
                    Self::DoubleLiteral(-n as f64)
                } else {
                    Self::LongLiteral(-n)
                }
            }
            _ => Self::UnaryExpression(
                UnaryOperator::Minus,
                Box::new(child.simplify_unary_minus_literals().at_pos(pos)),
            ),
        }
    }

    pub fn simplify_unary_minus_literals(self) -> Self {
        match self {
            Self::UnaryExpression(op, child) => {
                let x: ExpressionPos = *child;
                match op {
                    UnaryOperator::Minus => Self::unary_minus(x),
                    _ => Self::UnaryExpression(op, Self::simplify_unary_minus_pos(x)),
                }
            }
            Self::BinaryExpression(op, left, right, old_expression_type) => Self::BinaryExpression(
                op,
                Self::simplify_unary_minus_pos(*left),
                Self::simplify_unary_minus_pos(*right),
                old_expression_type,
            ),
            Self::Parenthesis(child) => Self::Parenthesis(Self::simplify_unary_minus_pos(*child)),
            Self::FunctionCall(name, args) => Self::FunctionCall(
                name,
                args.into_iter()
                    .map(|a| a.map(|x| x.simplify_unary_minus_literals()))
                    .collect(),
            ),
            _ => self,
        }
    }

    fn simplify_unary_minus_pos(child: ExpressionPos) -> Box<ExpressionPos> {
        let Positioned { element, pos } = child;
        let simplified = element.simplify_unary_minus_literals();
        Box::new(simplified.at_pos(pos))
    }

    /// Returns the name of this `Variable` or `Property` expression.
    /// For properties, this is the concatenated name of all elements in the property path.
    pub fn fold_name(&self) -> Option<Name> {
        match self {
            Self::Variable(n, _) => Some(n.clone()),
            Self::Property(left_side, property_name, _) => match left_side.fold_name() {
                Some(left_side_name) => left_side_name.try_concat_name(property_name.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn left_most_name(&self) -> Option<&Name> {
        match self {
            Self::Variable(n, _) | Self::FunctionCall(n, _) | Self::ArrayElement(n, _, _) => {
                Some(n)
            }
            Self::Property(left_side, _, _) => left_side.left_most_name(),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn var_unresolved(s: &str) -> Self {
        let name: Name = s.into();
        Self::Variable(name, VariableInfo::unresolved())
    }

    // TODO #[cfg(test)] but used by rusty_linter too
    pub fn var_resolved(s: &str) -> Self {
        let name: Name = s.into();
        let expression_type = name.expression_type();
        Self::Variable(name, VariableInfo::new_local(expression_type))
    }

    // TODO #[cfg(test)] but used by rusty_linter too
    pub fn var_user_defined(name: &str, type_name: &str) -> Self {
        Self::Variable(
            name.into(),
            VariableInfo::new_local(ExpressionType::UserDefined(type_name.into())),
        )
    }

    fn flip_multiply_plus(l_op: &Operator, r_op: &Operator) -> bool {
        (l_op.is_multiply_or_divide() || *l_op == Operator::Modulo) && r_op.is_plus_or_minus()
    }

    fn flip_plus_minus(l_op: &Operator, r_op: &Operator) -> bool {
        //
        //  A + B - C is parsed as
        //
        //      +
        //   A     -
        //        B C
        //
        // needs to flip into
        //
        //      -
        //   +    C
        //  A B
        l_op.is_plus_or_minus() && r_op.is_plus_or_minus()
    }

    fn flip_multiply_divide(l_op: &Operator, r_op: &Operator) -> bool {
        l_op.is_multiply_or_divide() && r_op.is_multiply_or_divide()
    }
}

// TODO #[deprecated]
pub trait ExpressionPosTrait {
    fn flip_binary(self) -> Self;

    fn simplify_unary_minus_literals(self) -> Self;

    fn apply_priority_order(self, right_side: Self, op: Operator, pos: Position) -> Self;

    fn binary_expr(self, op: Operator, right_side: Self, pos: Position) -> Self;

    fn apply_unary_priority_order(self, op: UnaryOperator, op_pos: Position) -> Self;
}

impl ExpressionPosTrait for ExpressionPos {
    /// Flips a binary expression.
    ///
    /// `A AND B OR C` would be parsed as `A AND (B OR C)` but needs to be `(A AND B) OR C`.
    fn flip_binary(self) -> Self {
        let Self { element, pos } = self;
        if let Expression::BinaryExpression(l_op, l_left, l_right, _) = element {
            let Self {
                element: r_element,
                pos: r_pos,
            } = *l_right;
            if let Expression::BinaryExpression(r_op, r_left, r_right, _) = r_element {
                let new_left = l_left.binary_expr(l_op, *r_left, pos);
                new_left.binary_expr(r_op, *r_right, r_pos)
            } else {
                panic!("should_flip_binary internal error")
            }
        } else {
            panic!("should_flip_binary internal error")
        }
    }

    fn simplify_unary_minus_literals(self) -> Self {
        self.map(|x| x.simplify_unary_minus_literals())
    }

    fn apply_priority_order(self, right_side: Self, op: Operator, pos: Position) -> Self {
        self.binary_expr(op, right_side, pos)
    }

    fn binary_expr(self, op: Operator, right_side: Self, pos: Position) -> Self {
        let result = Expression::BinaryExpression(
            op,
            Box::new(self),
            Box::new(right_side),
            ExpressionType::Unresolved,
        )
        .at_pos(pos);
        if result.should_flip_binary() {
            result.flip_binary()
        } else {
            result
        }
    }

    /// Applies the unary operator priority order.
    ///
    /// `NOT A AND B` would be parsed as `NOT (A AND B)`, needs to flip into `(NOT A) AND B`
    fn apply_unary_priority_order(self, op: UnaryOperator, op_pos: Position) -> Self {
        if self.should_flip_unary(op) {
            let Self { element, pos } = self;
            match element {
                Expression::BinaryExpression(r_op, r_left, r_right, _) => {
                    // apply the unary operator to the left of the binary expr
                    let new_left = Expression::UnaryExpression(op, r_left).at_pos(op_pos);
                    // and nest it as left inside a binary expr
                    new_left.binary_expr(r_op, *r_right, pos)
                }
                _ => panic!("should_flip_unary internal error"),
            }
        } else {
            Expression::UnaryExpression(op, Box::new(self)).at_pos(op_pos)
        }
    }
}

impl HasExpressionType for Expression {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::SingleLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::BangSingle),
            Self::DoubleLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::HashDouble),
            Self::StringLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::DollarString),
            Self::IntegerLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::PercentInteger),
            Self::LongLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::AmpersandLong),
            Self::Variable(
                _,
                VariableInfo {
                    expression_type, ..
                },
            )
            | Self::Property(_, _, expression_type)
            | Self::BinaryExpression(_, _, _, expression_type) => expression_type.clone(),
            Self::ArrayElement(
                _,
                args,
                VariableInfo {
                    expression_type, ..
                },
            ) => {
                if args.is_empty() {
                    // this is the entire array
                    ExpressionType::Array(Box::new(expression_type.clone()))
                } else {
                    // an element of the array
                    expression_type.clone()
                }
            }
            Self::FunctionCall(name, _) => name.expression_type(),
            Self::BuiltInFunctionCall(f, _) => ExpressionType::BuiltIn(f.into()),
            Self::UnaryExpression(_, c) | Self::Parenthesis(c) => c.expression_type(),
        }
    }
}

impl HasExpressionType for ExpressionPos {
    fn expression_type(&self) -> ExpressionType {
        self.element.expression_type()
    }
}

impl HasExpressionType for Box<ExpressionPos> {
    fn expression_type(&self) -> ExpressionType {
        self.as_ref().expression_type()
    }
}

pub trait ExpressionTrait {
    fn is_parenthesis(&self) -> bool;

    fn should_flip_unary(&self, op: UnaryOperator) -> bool;

    fn should_flip_binary(&self) -> bool;

    fn is_by_ref(&self) -> bool;
}

impl ExpressionTrait for Expression {
    fn is_parenthesis(&self) -> bool {
        matches!(self, Self::Parenthesis(_))
    }

    fn should_flip_unary(&self, op: UnaryOperator) -> bool {
        match self {
            Self::BinaryExpression(r_op, _, _, _) => op == UnaryOperator::Minus || r_op.is_binary(),
            _ => false,
        }
    }

    fn should_flip_binary(&self) -> bool {
        match self {
            Self::BinaryExpression(l_op, _, l_right, _) => match &l_right.element {
                Self::BinaryExpression(r_op, _, _, _) => {
                    l_op.is_arithmetic() && (r_op.is_relational() || r_op.is_binary())
                        || l_op.is_relational() && r_op.is_binary()
                        || *l_op == Operator::And && *r_op == Operator::Or
                        || Self::flip_multiply_plus(l_op, r_op)
                        || Self::flip_plus_minus(l_op, r_op)
                        || Self::flip_multiply_divide(l_op, r_op)
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn is_by_ref(&self) -> bool {
        matches!(
            self,
            Self::Variable(_, _) | Self::ArrayElement(_, _, _) | Self::Property(_, _, _)
        )
    }
}

impl ExpressionTrait for ExpressionPos {
    // needed by parser
    fn is_parenthesis(&self) -> bool {
        self.element.is_parenthesis()
    }

    fn should_flip_unary(&self, op: UnaryOperator) -> bool {
        self.element.should_flip_unary(op)
    }

    fn should_flip_binary(&self) -> bool {
        self.element.should_flip_binary()
    }

    fn is_by_ref(&self) -> bool {
        self.element.is_by_ref()
    }
}

impl ExpressionTrait for Box<ExpressionPos> {
    fn is_parenthesis(&self) -> bool {
        self.as_ref().is_parenthesis()
    }

    fn should_flip_unary(&self, op: UnaryOperator) -> bool {
        self.as_ref().should_flip_unary(op)
    }

    fn should_flip_binary(&self) -> bool {
        self.as_ref().should_flip_binary()
    }

    fn is_by_ref(&self) -> bool {
        self.as_ref().is_by_ref()
    }
}

/// `( expr [, expr]* )`
pub fn in_parenthesis_csv_expressions_non_opt(
    err_msg: &str,
) -> impl Parser<RcStringView, Output = Expressions, Error = ParseError> + '_ {
    in_parenthesis(csv_expressions_non_opt(err_msg)).no_incomplete()
}

/// Parses one or more expressions separated by comma.
/// FIXME Unlike csv_expressions, the first expression does not need a separator!
pub fn csv_expressions_non_opt(
    msg: &str,
) -> impl Parser<RcStringView, Output = Expressions, Error = ParseError> + use<'_> {
    csv_non_opt(expression_pos_p(), msg)
}

/// Parses one or more expressions separated by comma.
/// Trailing commas are not allowed.
/// Missing expressions are not allowed.
/// The first expression needs to be preceded by space or surrounded in parenthesis.
pub fn csv_expressions_first_guarded()
-> impl Parser<RcStringView, Output = Expressions, Error = ParseError> {
    ws_expr_pos_p().map(|first| vec![first]).and(
        comma()
            .and_keep_right(expression_pos_p().or_syntax_error("Expected: expression after comma"))
            .zero_or_more(),
        |mut l, mut r| {
            l.append(&mut r);
            l
        },
    )
}

lazy_parser!(
    pub fn expression_pos_p<I = RcStringView, Output = ExpressionPos, Error = ParseError> ;
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
pub fn ws_expr_pos_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
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
pub fn expr_pos_ws_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
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
pub fn ws_expr_pos_ws_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
    followed_by_ws(ws_expr_pos_p())
}

fn preceded_by_ws(
    parser: impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError>,
) -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
    guard::parser().and_keep_right(parser)
}

fn followed_by_ws(
    parser: impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError>,
) -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
    parser.then_with(
        |expr_pos| {
            let is_paren = expr_pos.is_parenthesis();
            conditionally_opt_whitespace(is_paren).no_incomplete()
        },
        |expr_pos, _| expr_pos,
    )
}

/// Parses an expression
fn eager_expression_pos_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError>
{
    binary_expression::parser()
}

mod single_or_double_literal {
    use rusty_pc::*;

    use crate::input::RcStringView;
    use crate::pc_specific::{WithPos, digits, dot, pound};
    use crate::{ParseError, *};

    // single ::= <digits> . <digits>
    // single ::= . <digits> (without leading zero)
    // double ::= <digits> . <digits> "#"
    // double ::= . <digits> "#"

    // TODO support more qualifiers besides '#'

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        // TODO this is difficult to understand
        opt_and_tuple(
            // read integer digits optionally (might start with . e.g. `.123`)
            digits(),
            // read dot and demand digits after decimal point
            // if dot is missing, the parser returns an empty result
            // the "deal breaker" is therefore the dot
            dot().and_keep_right(digits().no_incomplete()),
        )
        // and parse optionally a type qualifier such as `#`
        .and_opt_tuple(pound())
        // done parsing, flat map everything
        .flat_map(|input, ((opt_integer_digits, frac_digits), opt_pound)| {
            let left = opt_integer_digits
                .map(|token| token.to_str())
                .unwrap_or_else(|| "0".to_owned());
            let s = format!("{}.{}", left, frac_digits.as_str());
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
    use rusty_pc::*;

    use crate::input::RcStringView;
    use crate::pc_specific::*;
    use crate::{ParseError, *};

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        seq3(
            string_delimiter(),
            inside_string(),
            string_delimiter(),
            |_, token_list, _| Expression::StringLiteral(token_list_to_string(token_list)),
        )
        .with_pos()
    }

    fn string_delimiter() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
        any_token_of(TokenType::DoubleQuote)
    }

    fn inside_string() -> impl Parser<RcStringView, Output = TokenList, Error = ParseError> {
        any_token()
            .filter(|token| {
                !TokenType::DoubleQuote.matches(token) && !TokenType::Eol.matches(token)
            })
            .zero_or_more()
    }
}

mod integer_or_long_literal {
    use rusty_pc::*;
    use rusty_variant::{BitVec, BitVecIntOrLong, MAX_INTEGER, MAX_LONG};

    use crate::error::ParseError;
    use crate::input::RcStringView;
    use crate::pc_specific::{TokenType, WithPos, any_token};
    use crate::*;

    // result ::= <digits> | <hex-digits> | <oct-digits>
    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
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
            _ => panic!("Should not have processed {}", token.as_str()),
        };
        match res {
            Ok(expr) => Ok((input, expr)),
            Err(err) => Err((true, input, err)),
        }
    }

    fn process_dec(token: Token) -> Result<Expression, ParseError> {
        match token.to_str().parse::<u32>() {
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
        let mut s: String = token.to_str();
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
        let mut s: String = token.to_str();
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
    use std::collections::VecDeque;

    use rusty_pc::*;

    use crate::core::name::{name_with_dots_as_tokens, token_to_type_qualifier};
    use crate::input::RcStringView;
    use crate::pc_specific::{TokenType, WithPos};
    use crate::{ParseError, *};

    // variable ::= <identifier-with-dots>
    //           |  <identifier-with-dots> <type-qualifier>
    //           |  <keyword> "$"
    //
    // must not be followed by parenthesis (solved by ordering of parsers)
    //
    // if <identifier-with-dots> contains dots, it might be converted to a property expression
    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
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
            .map(|token| token.to_str())
            .collect();
        let mut result = Expression::Variable(
            Name::bare(BareName::new(property_names.pop_front().unwrap())),
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
    use rusty_pc::*;

    use crate::core::expression::expression_pos_p;
    use crate::core::name::name_with_dots_as_tokens;
    use crate::input::RcStringView;
    use crate::pc_specific::{WithPos, csv, in_parenthesis};
    use crate::{ParseError, *};

    // function_call ::= <function-name> "(" <expr>* ")"
    // function-name ::= <identifier-with-dots>
    //                |  <identifier-with-dots> <type-qualifier>
    //                |  <keyword> "$"
    //
    // Cannot invoke function with empty parenthesis, even if they don't have arguments.
    // However, it is allowed for arrays, so we parse it.
    //
    // A function can be qualified.

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
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
    use rusty_pc::*;

    use crate::core::name::{identifier, token_to_type_qualifier, type_qualifier};
    use crate::error::ParseError;
    use crate::input::RcStringView;
    use crate::pc_specific::{SpecificTrait, dot};
    use crate::*;

    // property ::= <expr> "." <property-name>
    // property-name ::= <identifier-without-dot>
    //                 | <identifier-without-dot> <type-qualifier>
    //
    // expr must not be qualified

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
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
                            BareName::new(name_token.to_str()),
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

    fn dot_properties()
    -> impl Parser<RcStringView, Output = Vec<(Token, Option<Token>)>, Error = ParseError> {
        dot_property().zero_or_more()
    }

    fn dot_property()
    -> impl Parser<RcStringView, Output = (Token, Option<Token>), Error = ParseError> {
        dot().and_keep_right(property().or_syntax_error("Expected: property name after dot"))
    }

    // cannot be followed by dot or type qualifier if qualified
    fn property() -> impl Parser<RcStringView, Output = (Token, Option<Token>), Error = ParseError>
    {
        identifier().and_opt_tuple(type_qualifier())
    }

    // can't use expression_pos_p because it will stack overflow
    fn base_expr_pos_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        // order is important, variable matches anything that function_call_or_array_element matches
        OrParser::new(vec![
            Box::new(super::function_call_or_array_element::parser()),
            Box::new(super::variable::parser()),
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
    use rusty_pc::Parser;

    use crate::built_ins::built_in_function_call_p;
    use crate::input::RcStringView;
    use crate::pc_specific::WithPos;
    use crate::{ParseError, *};

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        built_in_function_call_p().with_pos()
    }
}

mod binary_expression {
    use rusty_common::Positioned;
    use rusty_pc::*;

    use super::{
        built_in_function_call, expression_pos_p, guard, integer_or_long_literal, parenthesis, property, single_or_double_literal, string_literal, unary_expression
    };
    use crate::error::ParseError;
    use crate::input::RcStringView;
    use crate::pc_specific::{SpecificTrait, TokenType, WithPos, any_token, whitespace};
    use crate::*;

    // result ::= <non-bin-expr> <operator> <expr>
    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        BinaryExprParser
    }

    struct BinaryExprParser;

    impl Parser<RcStringView> for BinaryExprParser {
        type Output = ExpressionPos;
        type Error = ParseError;

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
            let (tokenizer, first) = Self::non_bin_expr().parse(tokenizer)?;

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

        fn non_bin_expr() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
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

        fn operator(
            is_paren: bool,
        ) -> impl Parser<RcStringView, Output = Positioned<Operator>, Error = ParseError> {
            opt_and_tuple(
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
    use rusty_common::Positioned;
    use rusty_pc::*;

    use crate::core::expression::{expression_pos_p, guard};
    use crate::input::RcStringView;
    use crate::pc_specific::{SpecificTrait, WithPos, keyword, minus_sign};
    use crate::{ParseError, *};

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        seq2(
            unary_op(),
            expression_pos_p().or_syntax_error("Expected: expression after unary operator"),
            |Positioned { element: op, pos }, expr| expr.apply_unary_priority_order(op, pos),
        )
    }

    fn unary_op()
    -> impl Parser<RcStringView, Output = Positioned<UnaryOperator>, Error = ParseError> {
        minus_sign()
            .map(|_| UnaryOperator::Minus)
            .or(keyword(Keyword::Not)
                .and_keep_right(guard::parser().no_incomplete())
                .map(|_| UnaryOperator::Not))
            .with_pos()
    }
}

mod parenthesis {
    use rusty_pc::*;

    use crate::core::expression::expression_pos_p;
    use crate::input::RcStringView;
    use crate::pc_specific::{SpecificTrait, WithPos, in_parenthesis};
    use crate::{ParseError, *};

    pub fn parser() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        in_parenthesis(
            expression_pos_p().or_syntax_error("Expected: expression inside parenthesis"),
        )
        .map(|child| Expression::Parenthesis(Box::new(child)))
        .with_pos()
    }
}

pub mod file_handle {
    //! Used by PRINT and built-ins

    use rusty_common::*;
    use rusty_pc::*;

    use crate::core::expression::ws_expr_pos_p;
    use crate::error::ParseError;
    use crate::input::RcStringView;
    use crate::pc_specific::*;
    use crate::*;

    pub fn file_handle_p()
    -> impl Parser<RcStringView, Output = Positioned<FileHandle>, Error = ParseError> {
        // # and digits
        // if # and 0 -> BadFileNameOrNumber
        // if # without digits -> SyntaxError (Expected: digits after #)
        any_token_of(TokenType::Pound)
            .with_pos()
            .and_tuple(any_token_of(TokenType::Digits).or_syntax_error("Expected: digits after #"))
            .flat_map(
                |input, (pound, digits)| match digits.to_str().parse::<u8>() {
                    Ok(d) if d > 0 => Ok((input, FileHandle::from(d).at_pos(pound.pos))),
                    _ => Err((true, input, ParseError::BadFileNameOrNumber)),
                },
            )
    }

    /// Parses a file handle ( e.g. `#1` ) as an integer literal expression.
    pub fn file_handle_as_expression_pos_p()
    -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        file_handle_p().map(|file_handle_pos| file_handle_pos.map(Expression::from))
    }

    pub fn guarded_file_handle_or_expression_p()
    -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        ws_file_handle().or(ws_expr_pos_p())
    }

    fn ws_file_handle() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
        whitespace().and_keep_right(file_handle_as_expression_pos_p())
    }
}

pub mod guard {
    use rusty_pc::*;

    use crate::ParseError;
    use crate::input::RcStringView;
    use crate::pc_specific::{TokenType, peek_token, whitespace};

    #[derive(Default)]
    pub enum Guard {
        #[default]
        Peeked,
        Whitespace,
    }

    /// `result ::= " " | "("`
    ///
    /// The "(" will be undone.
    pub fn parser() -> impl Parser<RcStringView, Output = Guard, Error = ParseError> {
        whitespace_guard().or(lparen_guard())
    }

    fn whitespace_guard() -> impl Parser<RcStringView, Output = Guard, Error = ParseError> {
        whitespace().map(|_| Guard::Whitespace)
    }

    fn lparen_guard() -> impl Parser<RcStringView, Output = Guard, Error = ParseError> {
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
    use rusty_common::*;

    use crate::error::ParseError;
    use crate::test_utils::*;
    use crate::{assert_expression, assert_literal_expression, assert_parser_err, expr, *};

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
                        vec![
                            Expression::BinaryExpression(
                                Operator::Plus,
                                Box::new(
                                    Expression::func("lbound", vec!["cards".as_var_expr(1, 20)])
                                        .at_rc(1, 13)
                                ),
                                Box::new(1.as_lit_expr(1, 29)),
                                ExpressionType::Unresolved
                            )
                            .at_rc(1, 27)
                        ]
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
                        vec![
                            Expression::BinaryExpression(
                                Operator::Minus,
                                Box::new("N".as_var_expr(1, 11)),
                                Box::new(1.as_lit_expr(1, 15)),
                                ExpressionType::Unresolved
                            )
                            .at_rc(1, 13)
                        ],
                    )
                    .at_rc(1, 7)
                ),
                Box::new(
                    Expression::func(
                        "Fib",
                        vec![
                            Expression::BinaryExpression(
                                Operator::Minus,
                                Box::new("N".as_var_expr(1, 24)),
                                Box::new(2.as_lit_expr(1, 28)),
                                ExpressionType::Unresolved
                            )
                            .at_rc(1, 26)
                        ],
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
                        vec![
                            Expression::UnaryExpression(
                                UnaryOperator::Minus,
                                Box::new("N".as_var_expr(1, 13)),
                            )
                            .at_rc(1, 12)
                        ],
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
