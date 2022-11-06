use crate::error::LintErrorPos;
use rusty_common::*;
use rusty_parser::*;

/// Visits the converted program and transforms it into a different program.
///
/// The default implementation of the trait simply clones all visited elements.
pub trait ExpressionReducer {
    fn visit_program(&mut self, program: Program) -> Result<Program, LintErrorPos> {
        let mut result: Program = vec![];
        for global_statement_pos in program {
            let x = self.visit_global_statement_pos(global_statement_pos)?;
            if let Some(s) = x {
                result.push(s);
            }
        }
        Ok(result)
    }

    fn visit_global_statement_pos(
        &mut self,
        global_statement_pos: GlobalStatementPos,
    ) -> Result<Option<GlobalStatementPos>, LintErrorPos> {
        let Positioned {
            element: global_statement,
            pos,
        } = global_statement_pos;
        self.visit_global_statement(global_statement)
            .map(|opt_global_statement| opt_global_statement.map(|t| t.at_pos(pos)))
            .patch_err_pos(&pos)
    }

    fn visit_global_statement(
        &mut self,
        global_statement: GlobalStatement,
    ) -> Result<Option<GlobalStatement>, LintErrorPos> {
        match global_statement {
            GlobalStatement::FunctionImplementation(f) => self
                .visit_function_implementation(f)
                .map(|x| Some(GlobalStatement::FunctionImplementation(x))),
            GlobalStatement::SubImplementation(s) => self
                .visit_sub_implementation(s)
                .map(|x| Some(GlobalStatement::SubImplementation(x))),
            GlobalStatement::Statement(s) => self
                .visit_filter_statement(s)
                .map(|opt_statement| opt_statement.map(GlobalStatement::Statement)),
            _ => Ok(None),
        }
    }

    fn visit_function_implementation(
        &mut self,
        f: FunctionImplementation,
    ) -> Result<FunctionImplementation, LintErrorPos> {
        Ok(FunctionImplementation {
            name: f.name,
            params: f.params,
            body: self.visit_statements(f.body)?,
            is_static: f.is_static,
        })
    }

    fn visit_sub_implementation(
        &mut self,
        s: SubImplementation,
    ) -> Result<SubImplementation, LintErrorPos> {
        Ok(SubImplementation {
            name: s.name,
            params: s.params,
            body: self.visit_statements(s.body)?,
            is_static: s.is_static,
        })
    }

    fn visit_statements(&mut self, statements: Statements) -> Result<Statements, LintErrorPos> {
        let mut result: Statements = vec![];
        for statement_pos in statements {
            let x = self.visit_statement_pos(statement_pos)?;
            if let Some(s) = x {
                result.push(s);
            }
        }
        Ok(result)
    }

    fn visit_statement_pos(
        &mut self,
        statement_pos: StatementPos,
    ) -> Result<Option<StatementPos>, LintErrorPos> {
        let Positioned {
            element: statement,
            pos,
        } = statement_pos;
        self.visit_filter_statement(statement)
            .map(|opt_statement| opt_statement.map(|x| x.at_pos(pos)))
            .patch_err_pos(&pos)
    }

    fn visit_filter_statement(&mut self, s: Statement) -> Result<Option<Statement>, LintErrorPos> {
        self.visit_map_statement(s).map(Some)
    }

    fn visit_map_statement(&mut self, s: Statement) -> Result<Statement, LintErrorPos> {
        match s {
            Statement::Assignment(left, right) => {
                self.visit_assignment(left, right)
                    .map(|(reduced_left, reduced_right)| {
                        Statement::Assignment(reduced_left, reduced_right)
                    })
            }
            Statement::Const(left, _) => panic!("Linter should have removed Const {:?}", left),
            Statement::SubCall(b, e) => self
                .visit_sub_call(b, e)
                .map(|(reduced_name, reduced_expr)| Statement::SubCall(reduced_name, reduced_expr)),
            Statement::BuiltInSubCall(b, e) => {
                self.visit_built_in_sub_call(b, e)
                    .map(|(reduced_name, reduced_expr)| {
                        Statement::BuiltInSubCall(reduced_name, reduced_expr)
                    })
            }
            Statement::Print(p) => self.visit_print(p).map(Statement::Print),
            Statement::IfBlock(i) => self.visit_if_block(i).map(Statement::IfBlock),
            Statement::SelectCase(s) => self.visit_select_case(s).map(Statement::SelectCase),
            Statement::ForLoop(f) => self.visit_for_loop(f).map(Statement::ForLoop),
            Statement::While(w) => self.visit_conditional_block(w).map(Statement::While),
            Statement::DoLoop(do_loop) => self.visit_do_loop(do_loop).map(Statement::DoLoop),
            Statement::Dim(dim_list) => self.visit_dim_list(dim_list).map(Statement::Dim),
            Statement::Redim(dim_list) => self.visit_dim_list(dim_list).map(Statement::Redim),
            Statement::OnError(_)
            | Statement::Label(_)
            | Statement::GoTo(_)
            | Statement::GoSub(_)
            | Statement::Resume(_)
            | Statement::Return(_)
            | Statement::Exit(_)
            | Statement::Comment(_)
            | Statement::End
            | Statement::System => Ok(s),
        }
    }

    fn visit_sub_call(
        &mut self,
        name: CaseInsensitiveString,
        args: Expressions,
    ) -> Result<(CaseInsensitiveString, Expressions), LintErrorPos> {
        Ok((name, self.visit_expressions(args)?))
    }

    fn visit_built_in_sub_call(
        &mut self,
        name: BuiltInSub,
        args: Expressions,
    ) -> Result<(BuiltInSub, Expressions), LintErrorPos> {
        Ok((name, self.visit_expressions(args)?))
    }

    fn visit_assignment(
        &mut self,
        name: Expression,
        v: ExpressionPos,
    ) -> Result<(Expression, ExpressionPos), LintErrorPos> {
        Ok((name, self.visit_expression_pos(v)?))
    }

    fn visit_for_loop(&mut self, f: ForLoop) -> Result<ForLoop, LintErrorPos> {
        let lower_bound = self.visit_expression_pos(f.lower_bound)?;
        let upper_bound = self.visit_expression_pos(f.upper_bound)?;
        let step = match f.step {
            Some(s) => Some(self.visit_expression_pos(s)?),
            None => None,
        };
        let statements = self.visit_statements(f.statements)?;
        Ok(ForLoop {
            lower_bound,
            upper_bound,
            step,
            statements,
            next_counter: f.next_counter,
            variable_name: f.variable_name,
        })
    }

    fn visit_if_block(&mut self, i: IfBlock) -> Result<IfBlock, LintErrorPos> {
        let if_block = self.visit_conditional_block(i.if_block)?;
        let else_if_blocks: Vec<ConditionalBlock> = i
            .else_if_blocks
            .into_iter()
            .map(|x| self.visit_conditional_block(x))
            .collect::<Result<Vec<ConditionalBlock>, LintErrorPos>>()?;
        let else_block: Option<Statements> = match i.else_block {
            Some(x) => Some(self.visit_statements(x)?),
            None => None,
        };
        Ok(IfBlock {
            if_block,
            else_if_blocks,
            else_block,
        })
    }

    fn visit_select_case(&mut self, s: SelectCase) -> Result<SelectCase, LintErrorPos> {
        let else_block: Option<Statements> = match s.else_block {
            Some(x) => Some(self.visit_statements(x)?),
            None => None,
        };
        let case_blocks: Vec<CaseBlock> = s
            .case_blocks
            .into_iter()
            .map(|x| self.visit_case_block(x))
            .collect::<Result<Vec<CaseBlock>, LintErrorPos>>()?;
        Ok(SelectCase {
            expr: self.visit_expression_pos(s.expr)?,
            case_blocks,
            else_block,
            inline_comments: s.inline_comments,
        })
    }

    fn visit_case_block(&mut self, s: CaseBlock) -> Result<CaseBlock, LintErrorPos> {
        let CaseBlock {
            expression_list,
            statements,
        } = s;
        let expression_list: Vec<CaseExpression> = expression_list
            .into_iter()
            .map(|case_expr| self.visit_case_expression(case_expr))
            .collect::<Result<Vec<CaseExpression>, LintErrorPos>>()?;
        Ok(CaseBlock {
            expression_list,
            statements: self.visit_statements(statements)?,
        })
    }

    fn visit_case_expression(&mut self, s: CaseExpression) -> Result<CaseExpression, LintErrorPos> {
        match s {
            CaseExpression::Simple(e) => self.visit_expression_pos(e).map(CaseExpression::Simple),
            CaseExpression::Is(op, e) => self
                .visit_expression_pos(e)
                .map(|x| CaseExpression::Is(op, x)),
            CaseExpression::Range(from, to) => self.visit_expression_pos(from).and_then(|x| {
                self.visit_expression_pos(to)
                    .map(|y| CaseExpression::Range(x, y))
            }),
        }
    }

    fn visit_do_loop(&mut self, do_loop: DoLoop) -> Result<DoLoop, LintErrorPos> {
        Ok(DoLoop {
            condition: self.visit_expression_pos(do_loop.condition)?,
            statements: self.visit_statements(do_loop.statements)?,
            position: do_loop.position,
            kind: do_loop.kind,
        })
    }

    fn visit_conditional_block(
        &mut self,
        c: ConditionalBlock,
    ) -> Result<ConditionalBlock, LintErrorPos> {
        Ok(ConditionalBlock {
            condition: self.visit_expression_pos(c.condition)?,
            statements: self.visit_statements(c.statements)?,
        })
    }

    fn visit_expression_pos(
        &mut self,
        expr_pos: ExpressionPos,
    ) -> Result<ExpressionPos, LintErrorPos> {
        let Positioned { element: expr, pos } = expr_pos;
        self.visit_expression(expr)
            .map(|x| x.at_pos(pos))
            .patch_err_pos(&pos)
    }

    fn visit_expression(&mut self, expression: Expression) -> Result<Expression, LintErrorPos> {
        Ok(expression)
    }

    fn visit_expressions(&mut self, args: Expressions) -> Result<Expressions, LintErrorPos> {
        args.into_iter()
            .map(|a| self.visit_expression_pos(a))
            .collect::<Result<Expressions, LintErrorPos>>()
    }

    fn visit_print(&mut self, print: Print) -> Result<Print, LintErrorPos> {
        Ok(Print {
            file_number: print.file_number,
            lpt1: print.lpt1,
            format_string: match print.format_string {
                Some(f) => self.visit_expression_pos(f).map(Some)?,
                _ => None,
            },
            args: self.visit_print_args(print.args)?,
        })
    }

    fn visit_print_args(
        &mut self,
        print_args: Vec<PrintArg>,
    ) -> Result<Vec<PrintArg>, LintErrorPos> {
        print_args
            .into_iter()
            .map(|a| self.visit_print_arg(a))
            .collect::<Result<Vec<PrintArg>, LintErrorPos>>()
    }

    fn visit_print_arg(&mut self, print_arg: PrintArg) -> Result<PrintArg, LintErrorPos> {
        match print_arg {
            PrintArg::Expression(e) => self.visit_expression_pos(e).map(PrintArg::Expression),
            _ => Ok(print_arg),
        }
    }

    fn visit_dim_list(&mut self, dim_list: DimList) -> Result<DimList, LintErrorPos> {
        let DimList { shared, variables } = dim_list;
        let converted_variables = self.dim_vars(variables)?;
        Ok(DimList {
            shared,
            variables: converted_variables,
        })
    }

    fn dim_vars(&mut self, dim_vars: DimVars) -> Result<DimVars, LintErrorPos> {
        dim_vars
            .into_iter()
            .map(|d| self.visit_dim_var_pos(d))
            .collect()
    }

    fn visit_dim_var_pos(&mut self, dim_var_pos: DimVarPos) -> Result<DimVarPos, LintErrorPos> {
        let DimVarPos {
            element:
                DimVar {
                    bare_name,
                    var_type: dim_type,
                },
            pos,
        } = dim_var_pos;
        let converted_dim_type = self.visit_dim_type(dim_type)?;
        Ok(DimVarPos {
            element: DimVar {
                bare_name,
                var_type: converted_dim_type,
            },
            pos,
        })
    }

    fn visit_dim_type(&mut self, dim_type: DimType) -> Result<DimType, LintErrorPos> {
        match dim_type {
            DimType::Array(array_dimensions, element_type) => {
                let converted_array_dimensions = self.visit_array_dimensions(array_dimensions)?;
                Ok(DimType::Array(converted_array_dimensions, element_type))
            }
            _ => Ok(dim_type),
        }
    }

    fn visit_array_dimensions(
        &mut self,
        array_dimensions: ArrayDimensions,
    ) -> Result<ArrayDimensions, LintErrorPos> {
        array_dimensions
            .into_iter()
            .map(|a| self.visit_array_dimension(a))
            .collect()
    }

    fn visit_array_dimension(
        &mut self,
        array_dimension: ArrayDimension,
    ) -> Result<ArrayDimension, LintErrorPos> {
        let ArrayDimension { lbound, ubound } = array_dimension;
        let converted_lbound = match lbound {
            Some(lbound) => Some(self.visit_expression_pos(lbound)?),
            None => None,
        };
        let converted_ubound = self.visit_expression_pos(ubound)?;
        Ok(ArrayDimension {
            lbound: converted_lbound,
            ubound: converted_ubound,
        })
    }
}
