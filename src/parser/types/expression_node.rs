use crate::built_ins::BuiltInFunction;
use crate::common::{
    AtLocation, CanCastTo, HasLocation, Locatable, Location, QError, QErrorNode, ToLocatableError,
};
use crate::parser::types::{Name, Operator, UnaryOperator};
use crate::parser::{ExpressionType, HasExpressionType, QualifiedName, TypeQualifier};
use crate::variant::{Variant, MIN_INTEGER, MIN_LONG};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    Constant(QualifiedName),
    Variable(Name, ExpressionType),
    FunctionCall(Name, ExpressionNodes),
    ArrayElement(
        // the name of the array (unqualified only for user defined types)
        Name,
        // the array indices
        Vec<ExpressionNode>,
        // the type of the elements
        ExpressionType,
    ),
    BuiltInFunctionCall(BuiltInFunction, Vec<ExpressionNode>),
    BinaryExpression(
        Operator,
        Box<ExpressionNode>,
        Box<ExpressionNode>,
        ExpressionType,
    ),
    UnaryExpression(UnaryOperator, Box<ExpressionNode>),
    Parenthesis(Box<ExpressionNode>),

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
    Property(Box<Expression>, Name, ExpressionType),
}

pub type ExpressionNode = Locatable<Expression>;
pub type ExpressionNodes = Vec<ExpressionNode>;

impl From<f32> for Expression {
    fn from(f: f32) -> Expression {
        Expression::SingleLiteral(f)
    }
}

impl From<f64> for Expression {
    fn from(f: f64) -> Expression {
        Expression::DoubleLiteral(f)
    }
}

impl From<String> for Expression {
    fn from(f: String) -> Expression {
        Expression::StringLiteral(f)
    }
}

impl From<&str> for Expression {
    fn from(f: &str) -> Expression {
        f.to_string().into()
    }
}

impl From<i32> for Expression {
    fn from(f: i32) -> Expression {
        Expression::IntegerLiteral(f)
    }
}

impl From<i64> for Expression {
    fn from(f: i64) -> Expression {
        Expression::LongLiteral(f)
    }
}

impl TryFrom<Variant> for Expression {
    type Error = QError;

    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::VSingle(f) => Ok(f.into()),
            Variant::VDouble(d) => Ok(d.into()),
            Variant::VString(s) => Ok(s.into()),
            Variant::VInteger(i) => Ok(i.into()),
            Variant::VLong(l) => Ok(l.into()),
            _ => Err(QError::InvalidConstant),
        }
    }
}

impl Expression {
    #[cfg(test)]
    pub fn var(s: &str) -> Self {
        let name: Name = s.into();
        Expression::Variable(name, ExpressionType::Unresolved)
    }

    #[cfg(test)]
    pub fn var_linted(s: &str) -> Self {
        let name: Name = s.into();
        let expression_type = name.expression_type();
        Expression::Variable(name, expression_type)
    }

    pub fn func(s: &str, args: ExpressionNodes) -> Self {
        let name: Name = s.into();
        Expression::FunctionCall(name, args)
    }

    fn unary_minus(child_node: ExpressionNode) -> Self {
        let Locatable {
            element: child,
            pos,
        } = child_node;
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
                Box::new(child.at(pos).simplify_unary_minus_literals()),
            ),
        }
    }

    pub fn simplify_unary_minus_literals(self) -> Self {
        match self {
            Self::UnaryExpression(op, child) => {
                let x: ExpressionNode = *child;
                match op {
                    UnaryOperator::Minus => Self::unary_minus(x),
                    _ => Self::UnaryExpression(op, Box::new(x.simplify_unary_minus_literals())),
                }
            }
            Self::BinaryExpression(op, left, right, old_expression_type) => {
                let x: ExpressionNode = *left;
                let y: ExpressionNode = *right;
                Self::BinaryExpression(
                    op,
                    Box::new(x.simplify_unary_minus_literals()),
                    Box::new(y.simplify_unary_minus_literals()),
                    old_expression_type,
                )
            }
            Self::Parenthesis(child) => {
                let x: ExpressionNode = *child;
                Self::Parenthesis(Box::new(x.simplify_unary_minus_literals()))
            }
            Self::FunctionCall(name, args) => Self::FunctionCall(
                name,
                args.into_iter()
                    .map(|a| a.map(|x| x.simplify_unary_minus_literals()))
                    .collect(),
            ),
            _ => self,
        }
    }

    pub fn is_parenthesis(&self) -> bool {
        match self {
            Self::Parenthesis(_) => true,
            _ => false,
        }
    }

    pub fn fold_name(self) -> Option<Name> {
        match self {
            Self::Variable(n, _) => Some(n),
            Self::Property(boxed_left_side, property_name, _) => {
                let left_side = *boxed_left_side;
                match left_side.fold_name() {
                    Some(left_side_name) => left_side_name.try_concat_name(property_name),
                    _ => None,
                }
            }
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

    pub fn binary(
        left: ExpressionNode,
        right: ExpressionNode,
        op: Operator,
    ) -> Result<Self, QErrorNode> {
        // get the types
        let t_left = left.expression_type();
        let t_right = right.expression_type();
        // get the cast type
        match t_left.cast_binary_op(t_right, op) {
            Some(type_definition) => Ok(Expression::BinaryExpression(
                op,
                Box::new(left),
                Box::new(right),
                type_definition,
            )),
            None => Err(QError::TypeMismatch).with_err_at(&right),
        }
    }

    #[cfg(test)]
    pub fn user_defined(name: &str, type_name: &str) -> Self {
        Expression::Variable(name.into(), ExpressionType::UserDefined(type_name.into()))
    }
}

impl ExpressionNode {
    pub fn simplify_unary_minus_literals(self) -> Self {
        self.map(|x| x.simplify_unary_minus_literals())
    }

    pub fn is_parenthesis(&self) -> bool {
        self.as_ref().is_parenthesis()
    }

    pub fn apply_priority_order(
        self,
        right_side: ExpressionNode,
        op: Operator,
        pos: Location,
    ) -> ExpressionNode {
        match right_side.as_ref() {
            Expression::BinaryExpression(r_op, r_left, r_right, _) => {
                let should_flip = op.is_arithmetic() && (r_op.is_relational() || r_op.is_binary())
                    || op.is_relational() && r_op.is_binary()
                    || op == Operator::And && *r_op == Operator::Or
                    || (op == Operator::Multiply || op == Operator::Divide)
                        && (*r_op == Operator::Plus || *r_op == Operator::Minus);
                if should_flip {
                    Expression::BinaryExpression(
                        *r_op,
                        Box::new(
                            Expression::BinaryExpression(
                                op,
                                Box::new(self),
                                r_left.clone(),
                                ExpressionType::Unresolved,
                            )
                            .at(pos),
                        ),
                        r_right.clone(),
                        ExpressionType::Unresolved,
                    )
                    .at(right_side.pos())
                } else {
                    Expression::BinaryExpression(
                        op,
                        Box::new(self),
                        Box::new(right_side),
                        ExpressionType::Unresolved,
                    )
                    .at(pos)
                }
            }
            _ => Expression::BinaryExpression(
                op,
                Box::new(self),
                Box::new(right_side),
                ExpressionType::Unresolved,
            )
            .at(pos),
        }
    }

    pub fn apply_unary_priority_order(self, op: UnaryOperator, pos: Location) -> ExpressionNode {
        match self.as_ref() {
            Expression::BinaryExpression(r_op, r_left, r_right, old_expression_type) => {
                let should_flip = op == UnaryOperator::Minus || r_op.is_binary();
                if should_flip {
                    Expression::BinaryExpression(
                        *r_op,
                        Box::new(Expression::UnaryExpression(op, r_left.clone()).at(pos)),
                        r_right.clone(),
                        old_expression_type.clone(),
                    )
                    .at(self.pos())
                } else {
                    Expression::UnaryExpression(op, Box::new(self)).at(pos)
                }
            }
            _ => Expression::UnaryExpression(op, Box::new(self)).at(pos),
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
            Self::Variable(_, expression_type)
            | Self::Property(_, _, expression_type)
            | Self::BinaryExpression(_, _, _, expression_type)
            | Self::ArrayElement(_, _, expression_type) => expression_type.clone(),
            Self::Constant(QualifiedName { qualifier, .. })
            | Self::FunctionCall(Name::Qualified(QualifiedName { qualifier, .. }), _) => {
                ExpressionType::BuiltIn(*qualifier)
            }
            Self::BuiltInFunctionCall(f, _) => ExpressionType::BuiltIn(f.into()),
            Self::UnaryExpression(_, c) | Self::Parenthesis(c) => c.as_ref().expression_type(),
            Self::FunctionCall(Name::Bare(_), _) => ExpressionType::Unresolved,
        }
    }
}

impl CanCastTo<TypeQualifier> for Expression {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        self.expression_type().can_cast_to(other)
    }
}

impl<T: HasExpressionType> CanCastTo<&T> for Expression {
    fn can_cast_to(&self, other: &T) -> bool {
        let other_type_definition = other.expression_type();
        self.expression_type().can_cast_to(&other_type_definition)
    }
}

impl From<QualifiedName> for Expression {
    fn from(var_name: QualifiedName) -> Self {
        let q = var_name.qualifier;
        Self::Variable(var_name.into(), ExpressionType::BuiltIn(q))
    }
}
