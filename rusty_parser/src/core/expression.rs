use rusty_common::*;
use rusty_pc::and::{KeepLeftCombiner, VecCombiner};
use rusty_pc::*;
use rusty_variant::{MIN_INTEGER, MIN_LONG};

use crate::core::opt_second_expression::conditionally_opt_whitespace;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::comma_ws;
use crate::{
    BuiltInFunction, ExpressionType, FileHandle, HasExpressionType, Name, Operator, ParserError, TypeQualifier, UnaryOperator, VariableInfo
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

    /// During parsing, arrays are also parsed as function calls.
    /// Only during linting can it be determined if it's an array or a function call.
    FunctionCall(Name, Expressions),

    /// Not a parser type, only at linting can we determine
    /// if it's a FunctionCall or an ArrayElement
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
    expectation: &str,
) -> impl Parser<StringView, Output = Expressions, Error = ParserError> + '_ {
    in_parenthesis(csv_expressions_non_opt(expectation)).to_fatal()
}

/// Parses one or more expressions separated by comma.
/// FIXME Unlike csv_expressions, the first expression does not need a separator!
pub fn csv_expressions_non_opt(
    expectation: &str,
) -> impl Parser<StringView, Output = Expressions, Error = ParserError> + use<'_> {
    csv_non_opt(expression_pos_p(), expectation)
}

/// Parses one or more expressions separated by comma.
/// Trailing commas are not allowed.
/// Missing expressions are not allowed.
/// The first expression needs to be preceded by space or surrounded in parenthesis.
pub fn csv_expressions_first_guarded()
-> impl Parser<StringView, Output = Expressions, Error = ParserError> {
    ws_expr_pos_p().map(|first| vec![first]).and(
        comma_ws()
            .and_keep_right(expression_pos_p().or_expected("expression after comma"))
            .zero_or_more(),
        VecCombiner,
    )
}

pub fn expression_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    lazy(eager_expression_pos_p)
}

/// Parses an expression that is either preceded by whitespace
/// or is a parenthesis expression.
///
/// ```text
/// <ws> <expr-not-in-parenthesis> |
/// <ws> <expr-in-parenthesis> |
/// <expr-in-parenthesis>
/// ```
pub fn ws_expr_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
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
pub fn expr_pos_ws_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
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
pub fn ws_expr_pos_ws_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    followed_by_ws(ws_expr_pos_p())
}

fn preceded_by_ws(
    parser: impl Parser<StringView, Output = ExpressionPos, Error = ParserError>,
) -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    guard::parser().and_keep_right(parser)
}

fn followed_by_ws(
    parser: impl Parser<StringView, Output = ExpressionPos, Error = ParserError>,
) -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    parser.then_with_in_context(
        conditionally_opt_whitespace(),
        |e| e.is_parenthesis(),
        KeepLeftCombiner,
    )
}

/// Parses an expression
fn eager_expression_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError>
{
    binary_expression::parser()
}

mod single_or_double_literal {
    use rusty_pc::and::opt_and_tuple;
    use rusty_pc::*;

    use crate::input::StringView;
    use crate::pc_specific::WithPos;
    use crate::tokens::{digits, dot, pound};
    use crate::{ParserError, *};

    // single ::= <digits> . <digits>
    // single ::= . <digits> (without leading zero)
    // double ::= <digits> . <digits> "#"
    // double ::= . <digits> "#"

    // TODO support more qualifiers besides '#'

    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        // TODO this is difficult to understand
        opt_and_tuple(
            // read integer digits optionally (might start with . e.g. `.123`)
            digits(),
            // read dot and demand digits after decimal point
            // if dot is missing, the parser returns an empty result
            // the "deal breaker" is therefore the dot
            dot().and_keep_right(digits().to_fatal()),
        )
        // and parse optionally a type qualifier such as `#`
        .and_opt_tuple(pound())
        // done parsing, flat map everything
        .and_then(|((opt_integer_digits, frac_digits), opt_pound)| {
            let left = opt_integer_digits
                .map(|token| token.to_string())
                .unwrap_or_else(|| "0".to_owned());
            let s = format!("{}.{}", left, frac_digits.as_str());
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
    use rusty_pc::many::StringManyCombiner;
    use rusty_pc::*;

    use crate::input::StringView;
    use crate::pc_specific::*;
    use crate::tokens::{MatchMode, TokenType, any_symbol_of, any_token_of};
    use crate::{ParserError, *};

    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        surround(
            string_delimiter(),
            inside_string(),
            string_delimiter(),
            SurroundMode::Mandatory,
        )
        .map(Expression::StringLiteral)
        .with_pos()
    }

    fn string_delimiter() -> impl Parser<StringView, Output = Token, Error = ParserError> {
        // TODO support ignoring token to avoid allocation
        any_symbol_of!('"')
    }

    fn inside_string() -> impl Parser<StringView, Output = String, Error = ParserError> {
        any_token_of!(
            types = TokenType::Eol ;
            symbols = '"' ;
            mode = MatchMode::Exclude)
        .many_allow_none(StringManyCombiner)
    }
}

mod integer_or_long_literal {
    use rusty_pc::*;
    use rusty_variant::{BitVec, BitVecIntOrLong, MAX_INTEGER, MAX_LONG};

    use crate::error::ParserError;
    use crate::input::StringView;
    use crate::pc_specific::WithPos;
    use crate::tokens::{TokenType, any_token_of};
    use crate::*;

    // result ::= <digits> | <hex-digits> | <oct-digits>
    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        any_token_of!(
            TokenType::Digits,
            TokenType::HexDigits,
            TokenType::OctDigits
        )
        .and_then(process_token)
        .with_pos()
    }

    fn process_token(token: Token) -> Result<Expression, ParserError> {
        match TokenType::from_token(&token) {
            TokenType::Digits => process_dec(token),
            TokenType::HexDigits => process_hex(token),
            TokenType::OctDigits => process_oct(token),
            _ => panic!("Should not have processed {}", token),
        }
    }

    fn process_dec(token: Token) -> Result<Expression, ParserError> {
        match token.to_string().parse::<u32>() {
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

    fn process_hex(token: Token) -> Result<Expression, ParserError> {
        // token text is &HFFFF or &H-FFFF
        let mut s: String = token.to_string();
        // remove &
        s.remove(0);
        // remove H
        s.remove(0);
        if s.starts_with('-') {
            Err(ParserError::Overflow)
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

    fn process_oct(token: Token) -> Result<Expression, ParserError> {
        let mut s: String = token.to_string();
        // remove &
        s.remove(0);
        // remove O
        s.remove(0);
        if s.starts_with('-') {
            Err(ParserError::Overflow)
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

    fn create_expression_from_bit_vec(bit_vec: BitVec) -> Result<Expression, ParserError> {
        bit_vec
            .convert_to_int_or_long_expr()
            .map(|x| match x {
                BitVecIntOrLong::Int(i) => Expression::IntegerLiteral(i),
                BitVecIntOrLong::Long(l) => Expression::LongLiteral(l),
            })
            .map_err(|_| ParserError::Overflow)
    }
}

// TODO consider nesting variable/function_call modules inside property as they are only used there
mod variable {
    use std::collections::VecDeque;

    use rusty_pc::*;

    use crate::core::name::{name_as_tokens_p, token_to_type_qualifier};
    use crate::input::StringView;
    use crate::pc_specific::WithPos;
    use crate::{ParserError, *};

    // variable ::= <identifier-with-dots>
    //           |  <identifier-with-dots> <type-qualifier>
    //           |  <keyword> "$"
    //
    // must not be followed by parenthesis (solved by ordering of parsers)
    //
    // if <identifier-with-dots> contains dots, it might be converted to a property expression
    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        name_as_tokens_p().map(map_to_expr).with_pos()
    }

    fn map_to_expr(name_as_tokens: NameAsTokens) -> Expression {
        if is_property_expr(&name_as_tokens) {
            map_to_property(name_as_tokens)
        } else {
            Expression::Variable(name_as_tokens.into(), VariableInfo::unresolved())
        }
    }

    fn is_property_expr(name_as_tokens: &NameAsTokens) -> bool {
        let (name_token, _) = name_as_tokens;
        let mut name_count = 1;
        let mut last_was_dot = false;

        // leading dot cannot happen
        debug_assert!(!name_token.as_str().starts_with('.'));

        for name in name_token.as_str().chars() {
            if '.' == name {
                if last_was_dot {
                    // two dots in a row
                    return false;
                } else {
                    last_was_dot = true;
                }
            } else {
                if last_was_dot {
                    name_count += 1;
                    last_was_dot = false;
                }
            }
        }
        // at least two names and no trailing dot
        name_count > 1 && !last_was_dot
    }

    fn map_to_property(name_as_tokens: NameAsTokens) -> Expression {
        let (name_token, opt_q_token) = name_as_tokens;
        let mut property_names: VecDeque<String> = name_token
            .as_str()
            .split('.')
            .map(|s| s.to_owned())
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
    use crate::core::name::name_as_tokens_p;
    use crate::input::StringView;
    use crate::pc_specific::{WithPos, csv, in_parenthesis};
    use crate::{ParserError, *};

    // function_call ::= <function-name> "(" <expr>* ")"
    // function-name ::= <identifier-with-dots>
    //                |  <identifier-with-dots> <type-qualifier>
    //                |  <keyword> "$"
    //
    // Cannot invoke function with empty parenthesis, even if they don't have arguments.
    // However, it is allowed for arrays, so we parse it.
    //
    // A function can be qualified.

    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        name_as_tokens_p()
            .and(
                in_parenthesis(csv(expression_pos_p()).or_default()),
                |name_as_tokens: NameAsTokens, arguments: Expressions| {
                    Expression::FunctionCall(name_as_tokens.into(), arguments)
                },
            )
            .with_pos()
    }
}

pub mod property {
    use rusty_pc::and::TupleCombiner;
    use rusty_pc::*;

    use crate::core::name::{name_as_tokens_p, token_to_type_qualifier};
    use crate::error::ParserError;
    use crate::input::StringView;
    use crate::pc_specific::OrExpected;
    use crate::tokens::dot;
    use crate::*;

    // property ::= <expr> "." <property-name>
    // property-name ::= <identifier-without-dot>
    //                 | <identifier-without-dot> <type-qualifier>
    //
    // expr must not be qualified

    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        base_expr_pos_p()
            .then_with_in_context(
                ctx_dot_property(),
                |first_expr_pos| is_qualified(&first_expr_pos.element),
                TupleCombiner,
            )
            .and_then(|(first_expr_pos, properties)| {
                // not possible to have properties for qualified first expr
                // therefore either we don't have properties
                // or if we do then the first expr is bare
                debug_assert!(properties.is_none() || !is_qualified(&first_expr_pos.element));

                match properties {
                    Some((property_name, opt_q_token)) => {
                        let text = property_name.as_str();
                        let mut it = text.split('.').peekable();
                        let mut result = first_expr_pos;
                        while let Some(name) = it.next() {
                            if name.is_empty() {
                                // detected something like X = Y(1).A..B
                                return Err(ParserError::expected("identifier").to_fatal());
                            }

                            let property_name = if it.peek().is_some() {
                                Name::bare(BareName::new(name.to_owned()))
                            } else {
                                Name::new(
                                    BareName::new(name.to_owned()),
                                    opt_q_token.as_ref().map(token_to_type_qualifier),
                                )
                            };

                            result = result.map(|prev_expr| {
                                Expression::Property(
                                    Box::new(prev_expr),
                                    property_name,
                                    ExpressionType::Unresolved,
                                )
                            });
                        }
                        Ok(result)
                    }
                    None => Ok(first_expr_pos),
                }
            })
    }

    /// Parses an optional `.property` after the first expression was parsed.
    /// The boolean context indicates whether the previously parsed expression
    /// was qualified or not.
    /// If it was qualified, we return Ok(None) without trying to parse,
    /// because qualified names can't have properties.
    fn ctx_dot_property()
    -> impl Parser<StringView, bool, Output = Option<NameAsTokens>, Error = ParserError>
    + SetContext<bool> {
        ctx_parser()
            .and_then(|was_first_expr_qualified| {
                if was_first_expr_qualified {
                    // fine, don't parse anything further
                    // i.e. A$(1).Name won't attempt to parse the .Name part
                    Ok(None)
                } else {
                    // it wasn't qualified, therefore let the dot_property continue
                    default_parse_error()
                }
            })
            .or(dot_property().no_context())
    }

    fn dot_property() -> impl Parser<StringView, Output = Option<NameAsTokens>, Error = ParserError>
    {
        dot()
            .and_keep_right(name_as_tokens_p().or_expected("property name after dot"))
            .to_option()
    }

    /// Returns a name expression which could be a function call (array)
    /// e.g. `A$(1)`,
    /// or a variable e.g. `A.B.C$` (which can be Variable or Property).
    /// can't use expression_pos_p because it will stack overflow
    fn base_expr_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        // order is important, variable matches anything that function_call_or_array_element matches
        OrParser::new(vec![
            Box::new(super::function_call_or_array_element::parser()),
            Box::new(super::variable::parser()),
        ])
    }

    pub fn is_qualified(expr: &Expression) -> bool {
        match expr {
            Expression::Variable(name, _)
            | Expression::FunctionCall(name, _)
            | Expression::Property(_, name, _) => !name.is_bare(),
            _ => {
                panic!("Unexpected property type {:?}", expr)
            }
        }
    }
}

mod built_in_function_call {
    use rusty_pc::Parser;

    use crate::built_ins::built_in_function_call_p;
    use crate::input::StringView;
    use crate::pc_specific::WithPos;
    use crate::{ParserError, *};

    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        built_in_function_call_p().with_pos()
    }
}

mod binary_expression {
    use rusty_common::Positioned;
    use rusty_pc::and::{TupleCombiner, opt_and_keep_right};
    use rusty_pc::*;

    use super::{
        built_in_function_call, expression_pos_p, guard, integer_or_long_literal, parenthesis, property, single_or_double_literal, string_literal, unary_expression
    };
    use crate::error::ParserError;
    use crate::input::StringView;
    use crate::pc_specific::{OrExpected, WithPos};
    use crate::tokens::{TokenType, any_token, whitespace_ignoring};
    use crate::*;

    // result ::= <non-bin-expr> <operator> <expr>
    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        non_bin_expr()
            .then_with_in_context(
                second_parser(),
                |first| first.is_parenthesis(),
                TupleCombiner,
            )
            .map(|(l, r)| match r {
                Some((op, r)) => l.apply_priority_order(r, op.element, op.pos),
                None => l,
            })
            .map(ExpressionPos::simplify_unary_minus_literals)
    }

    fn second_parser() -> impl Parser<
        StringView,
        bool,
        Output = Option<(Positioned<Operator>, ExpressionPos)>,
        Error = ParserError,
    > + SetContext<bool> {
        operator()
            .then_with_in_context(third_parser(), |op| is_keyword_op(op), TupleCombiner)
            .to_option()
    }

    fn is_keyword_op(op: &Positioned<Operator>) -> bool {
        op.element == Operator::And || op.element == Operator::Or || op.element == Operator::Modulo
    }

    fn third_parser()
    -> impl Parser<StringView, bool, Output = ExpressionPos, Error = ParserError> + SetContext<bool>
    {
        IifParser::new(
            guard::parser().to_fatal(),
            guard::parser().to_option().map(|_| ()),
        )
        .and_keep_right(right_side_expr().no_context())
    }

    /// Parses the right side expression, after having parsed the binary operator
    fn right_side_expr() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        // boxed breaks apart the recursive type evaluation
        expression_pos_p()
            .or_expected("expression after operator")
            .boxed()
    }

    fn non_bin_expr() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
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

    /// Parses an operator.
    /// The parameter indicates if the previously parsed expression was wrapped in
    /// parenthesis. If that is the case, leading whitespace is not required for
    /// keyword based operators.
    fn operator()
    -> impl Parser<StringView, bool, Output = Positioned<Operator>, Error = ParserError>
    + SetContext<bool> {
        IifParser::new(
            // no whitespace needed
            opt_and_keep_right(whitespace_ignoring(), operator_p()),
            // whitespace needed
            whitespace_ignoring()
                .and_keep_right(operator_p())
                .or(opt_and_keep_right(
                    whitespace_ignoring(),
                    symbol_operator_p(),
                )),
        )
    }

    /// Parses an operator.
    /// Does not check for leading whitespace, this needs to be done at the caller!
    fn operator_p() -> impl Parser<StringView, Output = Positioned<Operator>, Error = ParserError> {
        any_token().filter_map(map_token_to_operator).with_pos()
    }

    /// Parses a symbol operator (i.e. excludes keyword based operators).
    /// Does not check for leading whitespace, this needs to be done at the caller!
    fn symbol_operator_p()
    -> impl Parser<StringView, Output = Positioned<Operator>, Error = ParserError> {
        any_token()
            .filter_map(map_token_to_symbol_operator)
            .with_pos()
    }

    /// Maps the given token to an operator.
    fn map_token_to_operator(token: &Token) -> Option<Operator> {
        map_token_to_symbol_operator(token).or_else(|| map_token_to_keyword_operator(token))
    }

    /// Maps the given token to an operator, considering only operators
    /// that are based on symbols (i.e. excludes keywords).
    /// Symbol based operators do not require leading whitespace.
    fn map_token_to_symbol_operator(token: &Token) -> Option<Operator> {
        match TokenType::from_token(token) {
            TokenType::LessEquals => Some(Operator::LessOrEqual),
            TokenType::Less => Some(Operator::Less),
            TokenType::GreaterEquals => Some(Operator::GreaterOrEqual),
            TokenType::Greater => Some(Operator::Greater),
            TokenType::Equals => Some(Operator::Equal),
            TokenType::NotEquals => Some(Operator::NotEqual),
            TokenType::Symbol => match token.demand_single_char() {
                '+' => Some(Operator::Plus),
                '-' => Some(Operator::Minus),
                '*' => Some(Operator::Multiply),
                '/' => Some(Operator::Divide),
                _ => None,
            },
            _ => None,
        }
    }

    /// Maps the given token to an operator, considering only operators
    /// that are based on keywords (i.e. excludes symbols).
    /// Keyword based operators require leading whitespace.
    fn map_token_to_keyword_operator(token: &Token) -> Option<Operator> {
        match TokenType::from_token(token) {
            TokenType::Keyword => match Keyword::try_from(token.as_str()).unwrap() {
                Keyword::Mod => Some(Operator::Modulo),
                Keyword::And => Some(Operator::And),
                Keyword::Or => Some(Operator::Or),
                _ => None,
            },
            _ => None,
        }
    }
}

mod unary_expression {
    use rusty_common::Positioned;
    use rusty_pc::*;

    use crate::core::expression::{expression_pos_p, guard};
    use crate::input::StringView;
    use crate::pc_specific::{OrExpected, WithPos, keyword};
    use crate::tokens::minus_sign;
    use crate::{ParserError, *};

    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        seq2(
            unary_op(),
            expression_pos_p().or_expected("expression after unary operator"),
            |Positioned { element: op, pos }, expr| expr.apply_unary_priority_order(op, pos),
        )
    }

    fn unary_op() -> impl Parser<StringView, Output = Positioned<UnaryOperator>, Error = ParserError>
    {
        minus_sign()
            .map(|_| UnaryOperator::Minus)
            .or(keyword(Keyword::Not)
                .and_keep_right(guard::parser().to_fatal())
                .map(|_| UnaryOperator::Not))
            .with_pos()
    }
}

mod parenthesis {
    use rusty_pc::*;

    use crate::core::expression::expression_pos_p;
    use crate::input::StringView;
    use crate::pc_specific::{OrExpected, WithPos, in_parenthesis};
    use crate::{ParserError, *};

    pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        in_parenthesis(expression_pos_p().or_expected("expression inside parenthesis"))
            .map(|child| Expression::Parenthesis(Box::new(child)))
            .with_pos()
    }
}

pub mod file_handle {
    //! Used by PRINT and built-ins

    use rusty_common::*;
    use rusty_pc::*;

    use crate::core::expression::ws_expr_pos_p;
    use crate::error::ParserError;
    use crate::input::StringView;
    use crate::pc_specific::*;
    use crate::tokens::{TokenType, any_token_of, pound, whitespace_ignoring};
    use crate::*;

    pub fn file_handle_p()
    -> impl Parser<StringView, Output = Positioned<FileHandle>, Error = ParserError> {
        // # and digits
        // if # and 0 -> BadFileNameOrNumber
        // if # without digits -> SyntaxError (Expected: digits after #)
        pound()
            .with_pos()
            .and_tuple(any_token_of!(TokenType::Digits).or_expected("digits after #"))
            .and_then(|(pound, digits)| match digits.as_str().parse::<u8>() {
                Ok(d) if d > 0 => Ok(FileHandle::from(d).at_pos(pound.pos)),
                _ => Err(ParserError::BadFileNameOrNumber),
            })
    }

    /// Parses a file handle ( e.g. `#1` ) as an integer literal expression.
    pub fn file_handle_as_expression_pos_p()
    -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        file_handle_p().map(|file_handle_pos| file_handle_pos.map(Expression::from))
    }

    pub fn guarded_file_handle_or_expression_p()
    -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        ws_file_handle().or(ws_expr_pos_p())
    }

    fn ws_file_handle() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
        whitespace_ignoring().and_keep_right(file_handle_as_expression_pos_p())
    }
}

pub mod guard {
    use rusty_pc::*;

    use crate::ParserError;
    use crate::input::StringView;
    use crate::pc_specific::WithExpected;
    use crate::tokens::{any_symbol_of, any_token_of, whitespace_ignoring};

    /// `result ::= " " | "("`
    ///
    /// The "(" will be undone.
    pub fn parser() -> impl Parser<StringView, Output = (), Error = ParserError> {
        whitespace_guard()
            .or(lparen_guard())
            .with_expected_message("Expected: '(' or whitespace")
    }

    fn whitespace_guard() -> impl Parser<StringView, Output = (), Error = ParserError> {
        whitespace_ignoring()
    }

    fn lparen_guard() -> impl Parser<StringView, Output = (), Error = ParserError> {
        any_symbol_of!('(').map(|_| ()).peek()
    }
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

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
        assert_parser_err!("PRINT 1AND 2", expected("end-of-statement"));
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
        assert_parser_err!("PRINT 1OR 2", expected("end-of-statement"));
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
            assert_parser_err!(input, ParserError::BadFileNameOrNumber);
        }

        #[test]
        fn test_file_handle_overflow() {
            let input = "CLOSE #256";
            assert_parser_err!(input, ParserError::BadFileNameOrNumber);
        }

        #[test]
        fn test_file_handle_negative() {
            let input = "CLOSE #-1";
            assert_parser_err!(input, expected("digits after #"));
        }
    }

    mod hexadecimal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_parser_err!("PRINT &H-10", ParserError::Overflow);
            assert_parser_err!("PRINT &H100000000", ParserError::Overflow);
        }
    }

    mod octal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_parser_err!("PRINT &O-10", ParserError::Overflow);
            assert_parser_err!("PRINT &O40000000000", ParserError::Overflow);
        }
    }

    mod len {
        use super::*;

        #[test]
        fn len_in_print_must_be_unqualified() {
            let program = r#"PRINT LEN!("hello")"#;
            assert_parser_err!(program, expected("("), 1, 10);
        }

        #[test]
        fn len_in_assignment_must_be_unqualified() {
            let program = r#"A = LEN!("hello")"#;
            assert_parser_err!(program, expected("("), 1, 8);
        }
    }

    #[cfg(test)]
    mod name {
        use rusty_pc::Parser;

        use super::*;
        use crate::input::create_string_tokenizer;

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
        fn test_left_side_bare_array_cannot_have_consecutive_dots_in_properties() {
            let inputs = ["A(1).O..ops", "A(1).O..ops = 1"];
            for input in inputs {
                assert_parser_err!(input, "Expected: identifier");
            }
        }

        #[test]
        fn test_left_side_expected_equals() {
            let inputs = [
                "abc$",       // this one would expect 'variable=expression' in QBasic
                "abc$%",      // this one would expect 'variable=expression' in QBasic
                "abc%% = 42", // this one would expect 'variable=expression' in QBasic
                // trailing dot
                "A$.",
                "A$.B",
                "A$. = \"hello\"",
                // property of qualified array
                "A$(1).Oops",
                "A$(1).Oops = 42",
                // trailing dot on qualified array property
                "A(1).Suits$.",
                "A(1).Suits$. = \"hi\"",
                // extra qualifier on qualified array property
                "A(1).Suits$%",
                "A(1).Suits$% = 42",
            ];
            for input in inputs {
                assert_parser_err!(input, expected("="));
            }
        }

        #[test]
        fn test_right_side_expected_end_of_statement() {
            let inputs = [
                // double qualifier
                "Help A%%",
                // property of qualified array
                "Help A$(1).Oops",
                // trailing dot on qualified array property
                "Help A(1).Suits$.",
                // extra qualifier on qualified array property
                "Help A(1).Suits$%",
            ];
            for input in inputs {
                assert_parser_err!(input, expected("end-of-statement"));
            }
        }

        mod test_with_expression_parser {
            use rusty_pc::InputTrait;

            use super::*;

            #[test]
            fn test_double_qualifier() {
                let input = "A%%";
                let mut reader = create_string_tokenizer(input.to_owned());
                let mut parser = expression_pos_p();
                let expr = parser.parse(&mut reader).ok().unwrap();
                assert!(!reader.is_eof());
                let expr = expr.element();
                assert!(matches!(expr, Expression::Variable(_, _)));
                if let Expression::Variable(n, _) = expr {
                    assert_eq!(
                        n,
                        Name::qualified("A".into(), TypeQualifier::PercentInteger)
                    );
                }
            }
            #[test]
            fn test_trailing_dot_after_qualifier() {
                let input = "A$.";
                let mut reader = create_string_tokenizer(input.to_owned());
                let mut parser = expression_pos_p();
                let expr = parser.parse(&mut reader).ok().unwrap();
                assert!(!reader.is_eof());
                let expr = expr.element();
                assert!(matches!(expr, Expression::Variable(_, _)));
                if let Expression::Variable(n, _) = expr {
                    assert_eq!(n, Name::qualified("A".into(), TypeQualifier::DollarString));
                }
            }
            #[test]
            fn test_property_of_qualified_array() {
                let input = "A$(1).Oops";
                let mut reader = create_string_tokenizer(input.to_owned());
                let mut parser = expression_pos_p();
                let expr = parser.parse(&mut reader).ok().unwrap();
                assert!(!reader.is_eof());
                let expr = expr.element();
                assert!(matches!(expr, Expression::FunctionCall(_, _)));
                if let Expression::FunctionCall(n, _) = expr {
                    assert_eq!(n, Name::qualified("A".into(), TypeQualifier::DollarString));
                }
            }
        }
    }
}
