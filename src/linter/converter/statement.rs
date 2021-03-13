use super::converter::{Converter, ConverterImpl};
use crate::common::*;
use crate::linter::converter::context::{ExprContext, NameContext};
use crate::linter::converter::converter::ConverterWithImplicitVariables;
use crate::parser::{
    DimName, DoLoopNode, ExitObject, Expression, ExpressionType, FieldItem, FieldNode, GetPutNode,
    LSetNode, QualifiedNameNode, Statement, StatementNode, StatementNodes, TypeQualifier,
    VariableInfo,
};

// A statement can be expanded into multiple statements to convert implicitly
// declared variables into explicit.
// Example:
//      A = B + C
// becomes
//      DIM B
//      DIM C
//      DIM A
//      A = B + C

impl<'a> Converter<StatementNode, StatementNodes> for ConverterImpl<'a> {
    fn convert(&mut self, statement_node: StatementNode) -> Result<StatementNodes, QErrorNode> {
        let mut result: StatementNodes = vec![];
        self.process_statement_node(statement_node, &mut result)?;
        Ok(result)
    }
}

impl<'a> Converter<StatementNodes, StatementNodes> for ConverterImpl<'a> {
    fn convert(&mut self, statement_nodes: StatementNodes) -> Result<StatementNodes, QErrorNode> {
        let mut result: StatementNodes = vec![];
        for statement_node in statement_nodes {
            self.process_statement_node(statement_node, &mut result)?;
        }
        Ok(result)
    }
}

impl<'a> ConverterImpl<'a> {
    fn process_statement_node(
        &mut self,
        statement_node: StatementNode,
        result: &mut StatementNodes,
    ) -> Result<(), QErrorNode> {
        let (converted_statement_node, implicit_vars) =
            self.convert_and_collect_implicit_variables(statement_node)?;
        for implicit_var in implicit_vars {
            let Locatable {
                element: q_name,
                pos,
            } = implicit_var;
            result.push(Statement::Dim(DimName::from(q_name).into_list(pos)).at(pos));
        }
        if let Some(s) = converted_statement_node {
            result.push(s);
        }
        Ok(())
    }
}

impl<'a> ConverterWithImplicitVariables<StatementNode, Option<StatementNode>>
    for ConverterImpl<'a>
{
    fn convert_and_collect_implicit_variables(
        &mut self,
        statement_node: StatementNode,
    ) -> Result<(Option<StatementNode>, Vec<QualifiedNameNode>), QErrorNode> {
        let Locatable {
            element: statement,
            pos,
        } = statement_node;
        match statement {
            Statement::Assignment(n, e) => self.assignment(n.at(pos), e).map(|(x, y)| (Some(x), y)),
            Statement::Const(n, e) => self.context.on_const(n, e).map(|_| (None, vec![])),
            Statement::SubCall(n, args) => {
                self.sub_call(n.at(pos), args).map(|(x, y)| (Some(x), y))
            }
            Statement::BuiltInSubCall(built_in_sub, args) => {
                let (converted_args, implicits) =
                    self.context.on_expressions(args, ExprContext::Parameter)?;
                Ok((
                    Some(Statement::BuiltInSubCall(built_in_sub, converted_args).at(pos)),
                    implicits,
                ))
            }
            Statement::IfBlock(i) => self
                .convert_and_collect_implicit_variables(i)
                .map(|(i, implicit_vars)| (Some(Statement::IfBlock(i).at(pos)), implicit_vars)),
            Statement::SelectCase(s) => self
                .convert_and_collect_implicit_variables(s)
                .map(|(s, implicit_vars)| (Some(Statement::SelectCase(s).at(pos)), implicit_vars)),
            Statement::ForLoop(f) => self
                .convert_and_collect_implicit_variables(f)
                .map(|(f, implicit_vars)| (Some(Statement::ForLoop(f).at(pos)), implicit_vars)),
            Statement::While(c) => self
                .convert_and_collect_implicit_variables(c)
                .map(|(c, implicit_vars)| (Some(Statement::While(c).at(pos)), implicit_vars)),
            Statement::DoLoop(do_loop_node) => self
                .convert_and_collect_implicit_variables(do_loop_node)
                .map(|(x, implicit_vars)| (Some(Statement::DoLoop(x).at(pos)), implicit_vars)),
            Statement::Return(opt_label) => {
                if opt_label.is_some() && self.context.is_in_subprogram() {
                    // cannot have RETURN with explicit label inside subprogram
                    Err(QError::syntax_error("Illegal in subprogram")).with_err_at(pos)
                } else {
                    Ok((Some(Statement::Return(opt_label).at(pos)), vec![]))
                }
            }
            Statement::Resume(resume_option) => {
                if self.context.is_in_subprogram() {
                    Err(QError::syntax_error("Illegal in subprogram")).with_err_at(pos)
                } else {
                    Ok((Some(Statement::Resume(resume_option).at(pos)), vec![]))
                }
            }
            Statement::Exit(exit_object) => match self.context.get_name_context() {
                NameContext::Global => {
                    Err(QError::syntax_error("Illegal outside of subprogram")).with_err_at(pos)
                }
                NameContext::Sub => {
                    if exit_object == ExitObject::Sub {
                        Ok((Some(Statement::Exit(exit_object).at(pos)), vec![]))
                    } else {
                        Err(QError::syntax_error("Illegal inside sub")).with_err_at(pos)
                    }
                }
                NameContext::Function => {
                    if exit_object == ExitObject::Function {
                        Ok((Some(Statement::Exit(exit_object).at(pos)), vec![]))
                    } else {
                        Err(QError::syntax_error("Illegal inside function")).with_err_at(pos)
                    }
                }
            },
            Statement::Dim(dim_list) => self.convert_and_collect_implicit_variables(dim_list).map(
                |(dim_list, implicit_vars_in_array_dimensions)| {
                    (
                        Some(Statement::Dim(dim_list).at(pos)),
                        implicit_vars_in_array_dimensions,
                    )
                },
            ),
            Statement::Print(print_node) => self
                .convert_and_collect_implicit_variables(print_node)
                .map(|(p, implicit_vars)| (Some(Statement::Print(p).at(pos)), implicit_vars)),
            Statement::Field(field_node) => self
                .convert_and_collect_implicit_variables(field_node)
                .map(|(field_node, implicit_vars)| {
                    (Some(Statement::Field(field_node).at(pos)), implicit_vars)
                }),
            Statement::Get(get_node) => self.convert_and_collect_implicit_variables(get_node).map(
                |(get_node, implicit_vars)| (Some(Statement::Get(get_node).at(pos)), implicit_vars),
            ),
            Statement::Put(put_node) => self.convert_and_collect_implicit_variables(put_node).map(
                |(put_node, implicit_vars)| (Some(Statement::Put(put_node).at(pos)), implicit_vars),
            ),
            Statement::LSet(lset_node) => self
                .convert_and_collect_implicit_variables(lset_node)
                .map(|(lset_node, implicit_vars)| {
                    (Some(Statement::LSet(lset_node).at(pos)), implicit_vars)
                }),
            Statement::OnError(_)
            | Statement::Label(_)
            | Statement::GoTo(_)
            | Statement::GoSub(_)
            | Statement::Comment(_)
            | Statement::End
            | Statement::System => Ok((Some(statement.at(pos)), vec![])),
        }
    }
}

impl<'a> ConverterWithImplicitVariables<DoLoopNode, DoLoopNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        do_loop_node: DoLoopNode,
    ) -> Result<(DoLoopNode, Vec<QualifiedNameNode>), QErrorNode> {
        let DoLoopNode {
            condition,
            statements,
            position,
            kind,
        } = do_loop_node;
        let (condition, implicit_vars) = self
            .context
            .on_expression(condition, ExprContext::Default)?;
        let statements = self.convert(statements)?;
        Ok((
            DoLoopNode {
                condition,
                statements,
                position,
                kind,
            },
            implicit_vars,
        ))
    }
}

impl<'a> ConverterWithImplicitVariables<FieldNode, FieldNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        field_node: FieldNode,
    ) -> Result<(FieldNode, Vec<QualifiedNameNode>), QErrorNode> {
        let FieldNode {
            file_number,
            fields,
        } = field_node;
        let mut converted_fields: Vec<FieldItem> = vec![];
        let mut implicit_vars: Vec<QualifiedNameNode> = vec![];
        for FieldItem {
            width,
            name: Locatable { element: name, pos },
        } in fields
        {
            // convert width
            let (width, mut width_implicits) =
                self.context.on_expression(width, ExprContext::Default)?;
            implicit_vars.append(&mut width_implicits);
            // convert name
            let name_expr = Expression::Variable(name, VariableInfo::unresolved()).at(pos);
            let (
                Locatable {
                    element: converted_expr,
                    pos,
                },
                mut name_implicits,
            ) = self
                .context
                .on_expression(name_expr, ExprContext::Assignment)?;
            implicit_vars.append(&mut name_implicits);
            match converted_expr {
                Expression::Variable(
                    converted_name,
                    VariableInfo {
                        expression_type,
                        shared,
                    },
                ) => {
                    debug_assert!(!shared, "FIELD var should not be SHARED");
                    if expression_type != ExpressionType::BuiltIn(TypeQualifier::DollarString) {
                        return Err(QError::TypeMismatch).with_err_at(pos);
                    }
                    converted_fields.push(FieldItem {
                        width,
                        name: converted_name.at(pos),
                    });
                }
                _ => {
                    panic!(
                        "Unexpected result in converting FIELD variable at {:?}",
                        pos
                    );
                }
            }
        }
        Ok((
            FieldNode {
                file_number,
                fields: converted_fields,
            },
            implicit_vars,
        ))
    }
}

impl<'a> ConverterWithImplicitVariables<LSetNode, LSetNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        lset_node: LSetNode,
    ) -> Result<(LSetNode, Vec<QualifiedNameNode>), QErrorNode> {
        let LSetNode {
            name: Locatable { element: name, pos },
            expr,
        } = lset_node;
        let mut implicit_vars: Vec<QualifiedNameNode> = vec![];
        // convert expr
        let (expr, mut expr_implicits) = self.context.on_expression(expr, ExprContext::Default)?;
        implicit_vars.append(&mut expr_implicits);
        // convert name
        // TODO reuse this from FIELD
        let name_expr = Expression::Variable(name, VariableInfo::unresolved()).at(pos);
        let (
            Locatable {
                element: converted_expr,
                pos,
            },
            mut name_implicits,
        ) = self
            .context
            .on_expression(name_expr, ExprContext::Assignment)?;
        implicit_vars.append(&mut name_implicits);
        match converted_expr {
            Expression::Variable(
                converted_name,
                VariableInfo {
                    expression_type,
                    shared,
                },
            ) => {
                debug_assert!(!shared, "LSET var should not be SHARED");
                if expression_type != ExpressionType::BuiltIn(TypeQualifier::DollarString) {
                    return Err(QError::TypeMismatch).with_err_at(pos);
                }
                Ok((
                    LSetNode {
                        name: converted_name.at(pos),
                        expr,
                    },
                    implicit_vars,
                ))
            }
            _ => {
                panic!("Unexpected result in converting LSET variable at {:?}", pos);
            }
        }
    }
}

impl<'a> ConverterWithImplicitVariables<GetPutNode, GetPutNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        get_put_node: GetPutNode,
    ) -> Result<(GetPutNode, Vec<QualifiedNameNode>), QErrorNode> {
        let GetPutNode {
            file_number,
            record_number,
            variable,
        } = get_put_node;
        if variable.is_some() {
            unimplemented!();
        }
        let (record_number, implicits) = self
            .context
            .on_opt_expression(record_number, ExprContext::Default)?;
        Ok((
            GetPutNode {
                file_number,
                record_number,
                variable,
            },
            implicits,
        ))
    }
}
