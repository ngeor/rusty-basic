use crate::built_ins::BuiltInFunction;
use crate::common::{Locatable, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::type_resolver::TypeResolver;
use crate::linter::{Expression, ExpressionType, HasExpressionType};
use crate::parser;
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};
use std::convert::TryInto;

impl<'a> Converter<parser::Expression, Expression> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::Expression) -> Result<Expression, QErrorNode> {
        match a {
            parser::Expression::SingleLiteral(f) => Ok(Expression::SingleLiteral(f)),
            parser::Expression::DoubleLiteral(f) => Ok(Expression::DoubleLiteral(f)),
            parser::Expression::StringLiteral(f) => Ok(Expression::StringLiteral(f)),
            parser::Expression::IntegerLiteral(f) => Ok(Expression::IntegerLiteral(f)),
            parser::Expression::LongLiteral(f) => Ok(Expression::LongLiteral(f)),
            parser::Expression::VariableName(name_expr) => self
                .resolve_name_in_expression(&name_expr)
                .with_err_no_pos(),
            parser::Expression::FunctionCall(name_expr, args) => {
                let n = name_expr.clone();
                let converted_args = self.convert(args)?;
                let opt_built_in: Option<BuiltInFunction> = (&n).try_into().with_err_no_pos()?;
                match opt_built_in {
                    Some(b) => Ok(Expression::BuiltInFunctionCall(b, converted_args)),
                    None => {
                        // is it a function or an array element?
                        if self.context.is_array(&n) {
                            let dim_name = self
                                .context
                                .resolve_name_in_assignment(&n, &self.resolver)
                                .with_err_no_pos()?;
                            Ok(Expression::ArrayElement(dim_name, converted_args))
                        } else {
                            Ok(Expression::FunctionCall(
                                self.resolver.resolve_name(&n),
                                converted_args,
                            ))
                        }
                    }
                }
            }
            parser::Expression::Property(_, _) => todo!(),
            parser::Expression::BinaryExpression(op, l, r) => {
                // unbox them
                let unboxed_left = *l;
                let unboxed_right = *r;
                // convert them
                let converted_left = self.convert(unboxed_left)?;
                let converted_right = self.convert(unboxed_right)?;
                // cast
                Expression::binary(converted_left, converted_right, op)
            }
            parser::Expression::UnaryExpression(op, c) => {
                let unboxed_child = *c;
                let converted_child = self.convert(unboxed_child)?;
                match converted_child.expression_type() {
                    ExpressionType::BuiltIn(TypeQualifier::DollarString) => {
                        Err(QError::TypeMismatch).with_err_at(&converted_child)
                    }
                    ExpressionType::BuiltIn(_) => {
                        Ok(Expression::UnaryExpression(op, Box::new(converted_child)))
                    }
                    // user defined cannot be in unary expressions
                    _ => Err(QError::TypeMismatch).with_err_no_pos(),
                }
            }
            parser::Expression::Parenthesis(c) => {
                let unboxed_child = *c;
                let converted_child = self.convert(unboxed_child)?;
                Ok(Expression::Parenthesis(Box::new(converted_child)))
            }
        }
    }
}

impl<'a> ConverterImpl<'a> {
    fn resolve_name_in_expression(&mut self, n: &Name) -> Result<Expression, QError> {
        match self.context.resolve_expression(n, &self.resolver)? {
            Some(x) => Ok(x),
            None => match self.resolve_name_as_subprogram(n)? {
                Some(x) => Ok(x),
                None => self
                    .context
                    .resolve_missing_name_in_expression(n, &self.resolver),
            },
        }
    }

    fn resolve_name_as_subprogram(&mut self, name: &Name) -> Result<Option<Expression>, QError> {
        let bare_name: &BareName = name.as_ref();
        if self.subs.contains_key(bare_name) {
            // using the name of a sub as a variable expression
            Err(QError::DuplicateDefinition)
        } else if self.functions.contains_key(bare_name) {
            // if the function expects arguments, argument count mismatch
            let Locatable {
                element: (f_type, f_args),
                ..
            } = self.functions.get(bare_name).unwrap();
            if !f_args.is_empty() {
                Err(QError::ArgumentCountMismatch)
            } else if self.context.is_function_context(bare_name) {
                // We are inside a function that takes no args, and we're using again
                // the name of that function as an expression.
                // This can only work as a variable, otherwise we'll get infinite recursive call.
                //
                // Example:
                // Function Test
                //     INPUT Test
                // End Function
                //
                // Return None and let the next handler add it as a new variable
                Ok(None)
            } else {
                match name {
                    Name::Bare(b) => Ok(Some(Expression::FunctionCall(
                        QualifiedName::new(b.clone(), *f_type),
                        vec![],
                    ))),
                    Name::Qualified(QualifiedName {
                        bare_name,
                        qualifier,
                    }) => {
                        // if the function is a different type and the name is qualified of a different type, duplication definition
                        if f_type != qualifier {
                            Err(QError::DuplicateDefinition)
                        } else {
                            Ok(Some(Expression::FunctionCall(
                                QualifiedName::new(bare_name.clone(), *f_type),
                                vec![],
                            )))
                        }
                    }
                }
            }
        } else {
            Ok(None)
        }
    }
}
