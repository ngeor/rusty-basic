use crate::common::*;
use crate::linter::converter::context::ExprContext;
use crate::linter::converter::{Converter, ConverterImpl, ConverterWithImplicitVariables, R};
use crate::linter::{DimContext, NameContext};
use crate::parser::{DimName, DoLoopNode, ExitObject, Statement, StatementNode, StatementNodes};

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
    ) -> R<Option<StatementNode>> {
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
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
                } else {
                    Ok((Some(Statement::Return(opt_label).at(pos)), vec![]))
                }
            }
            Statement::Resume(resume_option) => {
                if self.context.is_in_subprogram() {
                    Err(QError::IllegalInSubFunction).with_err_at(pos)
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
            Statement::Dim(dim_list) => self.context.on_dim(dim_list, DimContext::Default).map(
                |(dim_list, implicit_vars_in_array_dimensions)| {
                    (
                        Some(Statement::Dim(dim_list).at(pos)),
                        implicit_vars_in_array_dimensions,
                    )
                },
            ),
            Statement::Redim(dim_list) => self.context.on_dim(dim_list, DimContext::Redim).map(
                |(dim_list, implicit_vars_in_array_dimensions)| {
                    (
                        Some(Statement::Redim(dim_list).at(pos)),
                        implicit_vars_in_array_dimensions,
                    )
                },
            ),
            Statement::Print(print_node) => self
                .convert_and_collect_implicit_variables(print_node)
                .map(|(p, implicit_vars)| (Some(Statement::Print(p).at(pos)), implicit_vars)),
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
    ) -> R<DoLoopNode> {
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
