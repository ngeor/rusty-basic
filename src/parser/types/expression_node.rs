use super::Name;
use crate::common::{AtLocation, FileHandle, HasLocation, Locatable, Location};
use crate::variant;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Operand {
    // relational
    Less,
    LessOrEqual,
    Equal,
    GreaterOrEqual,
    Greater,
    NotEqual,
    // arithmetic
    Plus,
    Minus,
    Multiply,
    Divide,
    // binary
    And,
    Or,
}

impl Operand {
    pub fn is_relational(&self) -> bool {
        match self {
            Self::Less
            | Self::LessOrEqual
            | Self::Equal
            | Self::GreaterOrEqual
            | Self::Greater
            | Self::NotEqual => true,
            _ => false,
        }
    }

    pub fn is_arithmetic(&self) -> bool {
        match self {
            Self::Plus | Self::Minus | Self::Multiply | Self::Divide => true,
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match self {
            Self::And | Self::Or => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnaryOperand {
    // Plus,
    Minus,
    Not,
}

pub type ArgumentNodes = Vec<ExpressionNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    #[allow(dead_code)]
    LongLiteral(i64),
    VariableName(Name),
    FunctionCall(Name, ArgumentNodes),
    BinaryExpression(Operand, Box<ExpressionNode>, Box<ExpressionNode>),
    UnaryExpression(UnaryOperand, Box<ExpressionNode>),
    Parenthesis(Box<ExpressionNode>),

    /// A file handle is used in built-in subs such as CLOSE #1
    FileHandle(FileHandle),
}

pub type ExpressionNode = Locatable<Expression>;

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
                if *n <= variant::MIN_INTEGER {
                    Self::LongLiteral(-n as i64)
                } else {
                    Self::IntegerLiteral(-n)
                }
            }
            Self::LongLiteral(n) => {
                if *n <= variant::MIN_LONG {
                    Self::DoubleLiteral(-n as f64)
                } else {
                    Self::LongLiteral(-n)
                }
            }
            _ => Self::UnaryExpression(
                UnaryOperand::Minus,
                Box::new(child.simplify_unary_minus_literals()),
            ),
        }
    }

    pub fn simplify_unary_minus_literals(self) -> Self {
        match self {
            Self::UnaryExpression(op, child) => {
                let x: ExpressionNode = *child;
                match op {
                    UnaryOperand::Minus => Self::unary_minus(x),
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
            Self::FunctionCall(name, args) => Self::FunctionCall(
                name,
                args.into_iter()
                    .map(|x| x.simplify_unary_minus_literals())
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
        op: Operand,
        pos: Location,
    ) -> ExpressionNode {
        match right_side.as_ref() {
            Expression::BinaryExpression(r_op, r_left, r_right) => {
                let should_flip = op.is_arithmetic() && (r_op.is_relational() || r_op.is_binary())
                    || op.is_relational() && r_op.is_binary()
                    || op == Operand::And && *r_op == Operand::Or
                    || (op == Operand::Multiply || op == Operand::Divide)
                        && (*r_op == Operand::Plus || *r_op == Operand::Minus);
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

    pub fn apply_unary_priority_order(self, op: UnaryOperand, pos: Location) -> ExpressionNode {
        match self.as_ref() {
            Expression::BinaryExpression(r_op, r_left, r_right) => {
                let should_flip = op == UnaryOperand::Minus || r_op.is_binary();
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
