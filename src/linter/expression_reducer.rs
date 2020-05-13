use super::error::*;
use super::types::*;
use crate::common::*;
use crate::parser::QualifiedName;

pub trait ExpressionReducer {
    fn visit_program(&self, p: ProgramNode) -> Result<ProgramNode, Error> {
        p.into_iter()
            .map(|t| self.visit_top_level_token_node(t))
            .collect()
    }

    fn visit_top_level_token_node(&self, t: TopLevelTokenNode) -> Result<TopLevelTokenNode, Error> {
        let (top_level_token, pos) = t.consume();
        self.visit_top_level_token(top_level_token)
            .with_pos(pos)
            .with_err_pos(pos)
    }

    fn visit_top_level_token(&self, t: TopLevelToken) -> Result<TopLevelToken, Error> {
        match t {
            TopLevelToken::FunctionImplementation(f) => self
                .visit_function_implementation(f)
                .map(|x| TopLevelToken::FunctionImplementation(x)),
            TopLevelToken::SubImplementation(s) => self
                .visit_sub_implementation(s)
                .map(|x| TopLevelToken::SubImplementation(x)),
            TopLevelToken::Statement(s) => {
                self.visit_statement(s).map(|x| TopLevelToken::Statement(x))
            }
        }
    }

    fn visit_function_implementation(
        &self,
        f: FunctionImplementation,
    ) -> Result<FunctionImplementation, Error> {
        Ok(FunctionImplementation {
            name: f.name,
            params: f.params,
            body: self.visit_statement_nodes(f.body)?,
        })
    }

    fn visit_sub_implementation(&self, s: SubImplementation) -> Result<SubImplementation, Error> {
        Ok(SubImplementation {
            name: s.name,
            params: s.params,
            body: self.visit_statement_nodes(s.body)?,
        })
    }

    fn visit_statement_nodes(&self, s: StatementNodes) -> Result<StatementNodes, Error> {
        s.into_iter()
            .map(|x| self.visit_statement_node(x))
            .collect()
    }

    fn visit_statement_node(&self, t: StatementNode) -> Result<StatementNode, Error> {
        let (statement, pos) = t.consume();
        self.visit_statement(statement)
            .with_pos(pos)
            .with_err_pos(pos)
    }

    fn visit_statement(&self, s: Statement) -> Result<Statement, Error> {
        match s {
            Statement::Assignment(left, right) => {
                self.visit_assignment(left, right)
                    .map(|(reduced_left, reduced_right)| {
                        Statement::Assignment(reduced_left, reduced_right)
                    })
            }
            Statement::Const(left, right) => self
                .visit_const(left, right)
                .map(|(reduced_left, reduced_right)| Statement::Const(reduced_left, reduced_right)),
            Statement::SubCall(b, e) => self
                .visit_sub_call(b, e)
                .map(|(reduced_name, reduced_expr)| Statement::SubCall(reduced_name, reduced_expr)),
            Statement::BuiltInSubCall(b, e) => {
                self.visit_built_in_sub_call(b, e)
                    .map(|(reduced_name, reduced_expr)| {
                        Statement::BuiltInSubCall(reduced_name, reduced_expr)
                    })
            }
            Statement::IfBlock(i) => self.visit_if_block(i).map(|x| Statement::IfBlock(x)),
            Statement::SelectCase(s) => self.visit_select_case(s).map(|x| Statement::SelectCase(x)),
            Statement::ForLoop(f) => self.visit_for_loop(f).map(|x| Statement::ForLoop(x)),
            Statement::While(w) => self.visit_conditional_block(w).map(|x| Statement::While(x)),
            Statement::ErrorHandler(label) => Ok(Statement::ErrorHandler(label)),
            Statement::Label(label) => Ok(Statement::Label(label)),
            Statement::GoTo(label) => Ok(Statement::GoTo(label)),
            Statement::SetReturnValue(expr) => self
                .visit_expression_node(expr)
                .map(|x| Statement::SetReturnValue(x)),
        }
    }

    fn visit_sub_call(
        &self,
        name: CaseInsensitiveString,
        args: Vec<ExpressionNode>,
    ) -> Result<(CaseInsensitiveString, Vec<ExpressionNode>), Error> {
        Ok((name, self.visit_expression_nodes(args)?))
    }

    fn visit_built_in_sub_call(
        &self,
        name: BuiltInSub,
        args: Vec<ExpressionNode>,
    ) -> Result<(BuiltInSub, Vec<ExpressionNode>), Error> {
        Ok((name, self.visit_expression_nodes(args)?))
    }

    fn visit_assignment(
        &self,
        name: QualifiedName,
        v: ExpressionNode,
    ) -> Result<(QualifiedName, ExpressionNode), Error> {
        Ok((name, self.visit_expression_node(v)?))
    }

    fn visit_for_loop(&self, f: ForLoopNode) -> Result<ForLoopNode, Error> {
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

    fn visit_if_block(&self, i: IfBlockNode) -> Result<IfBlockNode, Error> {
        let if_block = self.visit_conditional_block(i.if_block)?;
        let else_if_blocks: Vec<ConditionalBlockNode> = i
            .else_if_blocks
            .into_iter()
            .map(|x| self.visit_conditional_block(x))
            .collect::<Result<Vec<ConditionalBlockNode>, Error>>()?;
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

    fn visit_select_case(&self, s: SelectCaseNode) -> Result<SelectCaseNode, Error> {
        let else_block: Option<StatementNodes> = match s.else_block {
            Some(x) => Some(self.visit_statement_nodes(x)?),
            None => None,
        };
        let case_blocks: Vec<CaseBlockNode> = s
            .case_blocks
            .into_iter()
            .map(|x| self.visit_case_block(x))
            .collect::<Result<Vec<CaseBlockNode>, Error>>()?;
        Ok(SelectCaseNode {
            expr: self.visit_expression_node(s.expr)?,
            case_blocks,
            else_block,
        })
    }

    fn visit_case_block(&self, s: CaseBlockNode) -> Result<CaseBlockNode, Error> {
        Ok(CaseBlockNode {
            expr: self.visit_case_expression(s.expr)?,
            statements: self.visit_statement_nodes(s.statements)?,
        })
    }

    fn visit_case_expression(&self, s: CaseExpression) -> Result<CaseExpression, Error> {
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

    fn visit_conditional_block(
        &self,
        c: ConditionalBlockNode,
    ) -> Result<ConditionalBlockNode, Error> {
        Ok(ConditionalBlockNode {
            condition: self.visit_expression_node(c.condition)?,
            statements: self.visit_statement_nodes(c.statements)?,
        })
    }

    fn visit_const(
        &self,
        left: QNameNode,
        right: ExpressionNode,
    ) -> Result<(QNameNode, ExpressionNode), Error> {
        Ok((left, self.visit_expression_node(right)?))
    }

    fn visit_expression_node(&self, expr_node: ExpressionNode) -> Result<ExpressionNode, Error> {
        let (expr, pos) = expr_node.consume();
        self.visit_expression(expr).with_pos(pos).with_err_pos(pos)
    }

    fn visit_expression(&self, expression: Expression) -> Result<Expression, Error> {
        Ok(expression)
    }

    fn visit_expression_nodes(
        &self,
        args: Vec<ExpressionNode>,
    ) -> Result<Vec<ExpressionNode>, Error> {
        args.into_iter()
            .map(|a| self.visit_expression_node(a))
            .collect::<Result<Vec<ExpressionNode>, Error>>()
    }
}
