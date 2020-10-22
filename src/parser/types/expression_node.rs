use crate::common::{AtLocation, HasLocation, Locatable, Location};
use crate::parser::types::{NameExpr, Operator, UnaryOperator};
use crate::variant::{MIN_INTEGER, MIN_LONG};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    Name(NameExpr),
    BinaryExpression(Operator, Box<ExpressionNode>, Box<ExpressionNode>),
    UnaryExpression(UnaryOperator, Box<ExpressionNode>),
    Parenthesis(Box<ExpressionNode>),
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

impl Expression {
    fn unary_minus(child: ExpressionNode) -> Self {
        match child.as_ref() {
            Self::SingleLiteral(n) => Self::SingleLiteral(-n),
            Self::DoubleLiteral(n) => Self::DoubleLiteral(-n),
            Self::IntegerLiteral(n) => {
                if *n <= MIN_INTEGER {
                    Self::LongLiteral(-n as i64)
                } else {
                    Self::IntegerLiteral(-n)
                }
            }
            Self::LongLiteral(n) => {
                if *n <= MIN_LONG {
                    Self::DoubleLiteral(-n as f64)
                } else {
                    Self::LongLiteral(-n)
                }
            }
            _ => Self::UnaryExpression(
                UnaryOperator::Minus,
                Box::new(child.simplify_unary_minus_literals()),
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
            Self::BinaryExpression(op, left, right) => {
                let x: ExpressionNode = *left;
                let y: ExpressionNode = *right;
                Self::BinaryExpression(
                    op,
                    Box::new(x.simplify_unary_minus_literals()),
                    Box::new(y.simplify_unary_minus_literals()),
                )
            }
            Self::Parenthesis(child) => {
                let x: ExpressionNode = *child;
                Self::Parenthesis(Box::new(x.simplify_unary_minus_literals()))
            }
            Self::Name(name_expr) => Self::Name(NameExpr {
                bare_name: name_expr.bare_name,
                qualifier: name_expr.qualifier,
                arguments: name_expr.arguments.map(|x| {
                    x.into_iter()
                        .map(|a| a.simplify_unary_minus_literals())
                        .collect()
                }),
                elements: name_expr.elements,
            }),
            _ => self,
        }
    }

    pub fn is_parenthesis(&self) -> bool {
        match self {
            Self::Parenthesis(_) => true,
            _ => false,
        }
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
            Expression::BinaryExpression(r_op, r_left, r_right) => {
                let should_flip = op.is_arithmetic() && (r_op.is_relational() || r_op.is_binary())
                    || op.is_relational() && r_op.is_binary()
                    || op == Operator::And && *r_op == Operator::Or
                    || (op == Operator::Multiply || op == Operator::Divide)
                        && (*r_op == Operator::Plus || *r_op == Operator::Minus);
                if should_flip {
                    Expression::BinaryExpression(
                        *r_op,
                        Box::new(
                            Expression::BinaryExpression(op, Box::new(self), r_left.clone())
                                .at(pos),
                        ),
                        r_right.clone(),
                    )
                    .at(right_side.pos())
                } else {
                    Expression::BinaryExpression(op, Box::new(self), Box::new(right_side)).at(pos)
                }
            }
            _ => Expression::BinaryExpression(op, Box::new(self), Box::new(right_side)).at(pos),
        }
    }

    pub fn apply_unary_priority_order(self, op: UnaryOperator, pos: Location) -> ExpressionNode {
        match self.as_ref() {
            Expression::BinaryExpression(r_op, r_left, r_right) => {
                let should_flip = op == UnaryOperator::Minus || r_op.is_binary();
                if should_flip {
                    Expression::BinaryExpression(
                        *r_op,
                        Box::new(Expression::UnaryExpression(op, r_left.clone()).at(pos)),
                        r_right.clone(),
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
