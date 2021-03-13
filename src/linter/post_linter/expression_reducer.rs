use crate::built_ins::BuiltInSub;
use crate::common::*;
use crate::parser::*;

/// Visits the converted program and transforms it into a different program.
///
/// The default implementation of the trait simply clones all visited elements.
pub trait ExpressionReducer {
    fn visit_program(&mut self, p: ProgramNode) -> Result<ProgramNode, QErrorNode> {
        let mut result: ProgramNode = vec![];
        for top_level_token_node in p {
            let x = self.visit_top_level_token_node(top_level_token_node)?;
            if let Some(s) = x {
                result.push(s);
            }
        }
        Ok(result)
    }

    fn visit_top_level_token_node(
        &mut self,
        t: TopLevelTokenNode,
    ) -> Result<Option<TopLevelTokenNode>, QErrorNode> {
        let Locatable {
            element: top_level_token,
            pos,
        } = t;
        self.visit_top_level_token(top_level_token)
            .map(|opt_top_level_token| opt_top_level_token.map(|t| t.at(pos)))
            .patch_err_pos(pos)
    }

    fn visit_top_level_token(
        &mut self,
        t: TopLevelToken,
    ) -> Result<Option<TopLevelToken>, QErrorNode> {
        match t {
            TopLevelToken::FunctionImplementation(f) => self
                .visit_function_implementation(f)
                .map(|x| Some(TopLevelToken::FunctionImplementation(x))),
            TopLevelToken::SubImplementation(s) => self
                .visit_sub_implementation(s)
                .map(|x| Some(TopLevelToken::SubImplementation(x))),
            TopLevelToken::Statement(s) => self
                .visit_filter_statement(s)
                .map(|opt_statement| opt_statement.map(|x| TopLevelToken::Statement(x))),
            _ => Ok(None),
        }
    }

    fn visit_function_implementation(
        &mut self,
        f: FunctionImplementation,
    ) -> Result<FunctionImplementation, QErrorNode> {
        Ok(FunctionImplementation {
            name: f.name,
            params: f.params,
            body: self.visit_statement_nodes(f.body)?,
            is_static: f.is_static,
        })
    }

    fn visit_sub_implementation(
        &mut self,
        s: SubImplementation,
    ) -> Result<SubImplementation, QErrorNode> {
        Ok(SubImplementation {
            name: s.name,
            params: s.params,
            body: self.visit_statement_nodes(s.body)?,
            is_static: s.is_static,
        })
    }

    fn visit_statement_nodes(&mut self, s: StatementNodes) -> Result<StatementNodes, QErrorNode> {
        let mut result: StatementNodes = vec![];
        for statement_node in s {
            let x = self.visit_statement_node(statement_node)?;
            if let Some(s) = x {
                result.push(s);
            }
        }
        Ok(result)
    }

    fn visit_statement_node(
        &mut self,
        t: StatementNode,
    ) -> Result<Option<StatementNode>, QErrorNode> {
        let Locatable {
            element: statement,
            pos,
        } = t;
        self.visit_filter_statement(statement)
            .map(|opt_statement| opt_statement.map(|x| x.at(pos)))
            .patch_err_pos(pos)
    }

    fn visit_filter_statement(&mut self, s: Statement) -> Result<Option<Statement>, QErrorNode> {
        self.visit_map_statement(s).map(|s| Some(s))
    }

    fn visit_map_statement(&mut self, s: Statement) -> Result<Statement, QErrorNode> {
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
            Statement::Print(p) => self.visit_print_node(p).map(Statement::Print),
            Statement::Field(field_node) => self.visit_field_node(field_node).map(Statement::Field),
            Statement::Get(get_node) => self.visit_get_put_node(get_node).map(Statement::Get),
            Statement::Put(put_node) => self.visit_get_put_node(put_node).map(Statement::Put),
            Statement::IfBlock(i) => self.visit_if_block(i).map(Statement::IfBlock),
            Statement::SelectCase(s) => self.visit_select_case(s).map(Statement::SelectCase),
            Statement::ForLoop(f) => self.visit_for_loop(f).map(Statement::ForLoop),
            Statement::While(w) => self.visit_conditional_block(w).map(Statement::While),
            Statement::DoLoop(do_loop_node) => {
                self.visit_do_loop(do_loop_node).map(Statement::DoLoop)
            }
            Statement::Dim(_)
            | Statement::OnError(_)
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
        args: Vec<ExpressionNode>,
    ) -> Result<(CaseInsensitiveString, Vec<ExpressionNode>), QErrorNode> {
        Ok((name, self.visit_expression_nodes(args)?))
    }

    fn visit_built_in_sub_call(
        &mut self,
        name: BuiltInSub,
        args: Vec<ExpressionNode>,
    ) -> Result<(BuiltInSub, Vec<ExpressionNode>), QErrorNode> {
        Ok((name, self.visit_expression_nodes(args)?))
    }

    fn visit_assignment(
        &mut self,
        name: Expression,
        v: ExpressionNode,
    ) -> Result<(Expression, ExpressionNode), QErrorNode> {
        Ok((name, self.visit_expression_node(v)?))
    }

    fn visit_for_loop(&mut self, f: ForLoopNode) -> Result<ForLoopNode, QErrorNode> {
        let lower_bound = self.visit_expression_node(f.lower_bound)?;
        let upper_bound = self.visit_expression_node(f.upper_bound)?;
        let step = match f.step {
            Some(s) => Some(self.visit_expression_node(s)?),
            None => None,
        };
        let statements = self.visit_statement_nodes(f.statements)?;
        Ok(ForLoopNode {
            lower_bound,
            upper_bound,
            step,
            statements,
            next_counter: f.next_counter,
            variable_name: f.variable_name,
        })
    }

    fn visit_if_block(&mut self, i: IfBlockNode) -> Result<IfBlockNode, QErrorNode> {
        let if_block = self.visit_conditional_block(i.if_block)?;
        let else_if_blocks: Vec<ConditionalBlockNode> = i
            .else_if_blocks
            .into_iter()
            .map(|x| self.visit_conditional_block(x))
            .collect::<Result<Vec<ConditionalBlockNode>, QErrorNode>>()?;
        let else_block: Option<StatementNodes> = match i.else_block {
            Some(x) => Some(self.visit_statement_nodes(x)?),
            None => None,
        };
        Ok(IfBlockNode {
            if_block,
            else_if_blocks,
            else_block,
        })
    }

    fn visit_select_case(&mut self, s: SelectCaseNode) -> Result<SelectCaseNode, QErrorNode> {
        let else_block: Option<StatementNodes> = match s.else_block {
            Some(x) => Some(self.visit_statement_nodes(x)?),
            None => None,
        };
        let case_blocks: Vec<CaseBlockNode> = s
            .case_blocks
            .into_iter()
            .map(|x| self.visit_case_block(x))
            .collect::<Result<Vec<CaseBlockNode>, QErrorNode>>()?;
        Ok(SelectCaseNode {
            expr: self.visit_expression_node(s.expr)?,
            case_blocks,
            else_block,
            inline_comments: s.inline_comments,
        })
    }

    fn visit_case_block(&mut self, s: CaseBlockNode) -> Result<CaseBlockNode, QErrorNode> {
        let CaseBlockNode {
            expression_list,
            statements,
        } = s;
        let expression_list: Vec<CaseExpression> = expression_list
            .into_iter()
            .map(|case_expr| self.visit_case_expression(case_expr))
            .collect::<Result<Vec<CaseExpression>, QErrorNode>>()?;
        Ok(CaseBlockNode {
            expression_list,
            statements: self.visit_statement_nodes(statements)?,
        })
    }

    fn visit_case_expression(&mut self, s: CaseExpression) -> Result<CaseExpression, QErrorNode> {
        match s {
            CaseExpression::Simple(e) => self
                .visit_expression_node(e)
                .map(|x| CaseExpression::Simple(x)),
            CaseExpression::Is(op, e) => self
                .visit_expression_node(e)
                .map(|x| CaseExpression::Is(op, x)),
            CaseExpression::Range(from, to) => self.visit_expression_node(from).and_then(|x| {
                self.visit_expression_node(to)
                    .map(|y| CaseExpression::Range(x, y))
            }),
        }
    }

    fn visit_do_loop(&mut self, do_loop: DoLoopNode) -> Result<DoLoopNode, QErrorNode> {
        Ok(DoLoopNode {
            condition: self.visit_expression_node(do_loop.condition)?,
            statements: self.visit_statement_nodes(do_loop.statements)?,
            position: do_loop.position,
            kind: do_loop.kind,
        })
    }

    fn visit_conditional_block(
        &mut self,
        c: ConditionalBlockNode,
    ) -> Result<ConditionalBlockNode, QErrorNode> {
        Ok(ConditionalBlockNode {
            condition: self.visit_expression_node(c.condition)?,
            statements: self.visit_statement_nodes(c.statements)?,
        })
    }

    fn visit_expression_node(
        &mut self,
        expr_node: ExpressionNode,
    ) -> Result<ExpressionNode, QErrorNode> {
        let Locatable { element: expr, pos } = expr_node;
        self.visit_expression(expr)
            .map(|x| x.at(pos))
            .patch_err_pos(pos)
    }

    fn visit_expression(&mut self, expression: Expression) -> Result<Expression, QErrorNode> {
        Ok(expression)
    }

    fn visit_expression_nodes(
        &mut self,
        args: Vec<ExpressionNode>,
    ) -> Result<Vec<ExpressionNode>, QErrorNode> {
        args.into_iter()
            .map(|a| self.visit_expression_node(a))
            .collect::<Result<Vec<ExpressionNode>, QErrorNode>>()
    }

    fn visit_print_node(&mut self, print_node: PrintNode) -> Result<PrintNode, QErrorNode> {
        Ok(PrintNode {
            file_number: print_node.file_number,
            lpt1: print_node.lpt1,
            format_string: match print_node.format_string {
                Some(f) => self.visit_expression_node(f).map(|e| Some(e))?,
                _ => None,
            },
            args: self.visit_print_args(print_node.args)?,
        })
    }

    fn visit_print_args(&mut self, print_args: Vec<PrintArg>) -> Result<Vec<PrintArg>, QErrorNode> {
        print_args
            .into_iter()
            .map(|a| self.visit_print_arg(a))
            .collect::<Result<Vec<PrintArg>, QErrorNode>>()
    }

    fn visit_print_arg(&mut self, print_arg: PrintArg) -> Result<PrintArg, QErrorNode> {
        match print_arg {
            PrintArg::Expression(e) => self
                .visit_expression_node(e)
                .map(|converted_expr_node| PrintArg::Expression(converted_expr_node)),
            _ => Ok(print_arg),
        }
    }

    fn visit_field_node(&mut self, field_node: FieldNode) -> Result<FieldNode, QErrorNode> {
        Ok(field_node)
    }

    fn visit_get_put_node(&mut self, get_put_node: GetPutNode) -> Result<GetPutNode, QErrorNode> {
        Ok(get_put_node)
    }
}
