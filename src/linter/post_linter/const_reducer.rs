use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::linter::post_linter::expression_reducer::ExpressionReducer;
use crate::parser::{
    Expression, FunctionImplementation, Name, NameNode, QualifiedName, Statement, SubImplementation,
};
use crate::variant::Variant;
use std::collections::HashMap;

pub struct ConstReducer {
    global_constants: HashMap<QualifiedName, Variant>,
    local_constants: HashMap<QualifiedName, Variant>,
    in_sub_program: bool,
}

impl ConstReducer {
    pub fn new() -> Self {
        Self {
            global_constants: HashMap::new(),
            local_constants: HashMap::new(),
            in_sub_program: false,
        }
    }
}

impl ExpressionReducer for ConstReducer {
    fn visit_function_implementation(
        &mut self,
        f: FunctionImplementation,
    ) -> Result<FunctionImplementation, QErrorNode> {
        self.local_constants = HashMap::new();
        self.in_sub_program = true;
        let result = FunctionImplementation {
            name: f.name,
            params: f.params,
            body: self.visit_statement_nodes(f.body)?,
        };
        self.in_sub_program = false;
        Ok(result)
    }

    fn visit_sub_implementation(
        &mut self,
        s: SubImplementation,
    ) -> Result<SubImplementation, QErrorNode> {
        self.local_constants = HashMap::new();
        self.in_sub_program = true;
        let result = SubImplementation {
            name: s.name,
            params: s.params,
            body: self.visit_statement_nodes(s.body)?,
        };
        self.in_sub_program = false;
        Ok(result)
    }

    fn visit_filter_statement(&mut self, s: Statement) -> Result<Option<Statement>, QErrorNode> {
        let reduced = self.visit_map_statement(s)?;
        match reduced {
            Statement::Const(_, _, _) => Ok(None),
            _ => Ok(Some(reduced)),
        }
    }

    fn visit_const(
        &mut self,
        left: NameNode,
        right: Variant,
    ) -> Result<(NameNode, Variant), QErrorNode> {
        if let Name::Qualified(qualified_name) = left.as_ref() {
            if self.in_sub_program {
                self.local_constants
                    .insert(qualified_name.clone(), right.clone());
            } else {
                self.global_constants
                    .insert(qualified_name.clone(), right.clone());
            }
            Ok((left, right))
        } else {
            panic!("Unexpected bare constant {:?}", left)
        }
    }

    fn visit_expression(&mut self, expression: Expression) -> Result<Expression, QErrorNode> {
        match expression {
            Expression::Constant(qualified_name) => {
                // replace the constant with its value as a literal
                let mut opt_v = if self.in_sub_program {
                    self.local_constants.get(&qualified_name)
                } else {
                    None
                };
                if opt_v.is_none() {
                    // fall back to global constant
                    opt_v = self.global_constants.get(&qualified_name);
                }
                match opt_v {
                    Some(v) => match v {
                        Variant::VSingle(f) => Ok(Expression::SingleLiteral(*f)),
                        Variant::VDouble(f) => Ok(Expression::DoubleLiteral(*f)),
                        Variant::VInteger(f) => Ok(Expression::IntegerLiteral(*f)),
                        Variant::VLong(f) => Ok(Expression::LongLiteral(*f)),
                        Variant::VString(s) => Ok(Expression::StringLiteral(s.clone())),
                        _ => Err(QError::InvalidConstant).with_err_no_pos(),
                    },
                    _ => {
                        // should not happen
                        Err(QError::InvalidConstant).with_err_no_pos()
                    }
                }
            }
            _ => Ok(expression),
        }
    }
}
