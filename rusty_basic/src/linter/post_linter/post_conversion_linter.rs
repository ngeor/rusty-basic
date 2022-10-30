use rusty_common::*;
use rusty_parser::*;

/// Invoked after the conversion to fully typed program.
/// The default implementation of the trait simply visits all program elements.
///
/// The PostConversionLinter does not modify the elements it visits, which is
/// why it works with references.
///
/// Methods return Ok(()) to indicate an element passes the check or
/// Err() to indicate a problem.
pub trait PostConversionLinter {
    fn visit_program(&mut self, p: &ProgramNode) -> Result<(), QErrorNode> {
        // in case of overriding visit_program, use visit_top_level_token_nodes to call the default functionality
        self.visit_top_level_token_nodes(p)
    }

    fn visit_top_level_token_nodes(&mut self, p: &ProgramNode) -> Result<(), QErrorNode> {
        p.iter()
            .try_for_each(|t| self.visit_top_level_token_node(t))
    }

    fn visit_top_level_token_node(&mut self, t: &TopLevelTokenNode) -> Result<(), QErrorNode> {
        self.visit_top_level_token(t.as_ref()).patch_err_pos(t)
    }

    fn visit_top_level_token(&mut self, t: &TopLevelToken) -> Result<(), QErrorNode> {
        match t {
            TopLevelToken::FunctionImplementation(f) => self.visit_function_implementation(f),
            TopLevelToken::SubImplementation(s) => self.visit_sub_implementation(s),
            TopLevelToken::Statement(s) => self.visit_statement(s),
            _ => Ok(()),
        }
    }

    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&f.body)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&s.body)
    }

    fn visit_statement_nodes(&mut self, s: &StatementNodes) -> Result<(), QErrorNode> {
        s.iter().try_for_each(|x| self.visit_statement_node(x))
    }

    fn visit_statement_node(&mut self, t: &StatementNode) -> Result<(), QErrorNode> {
        self.visit_statement(t.as_ref()).patch_err_pos(t)
    }

    fn visit_statement(&mut self, s: &Statement) -> Result<(), QErrorNode> {
        match s {
            Statement::Assignment(left, right) => self.visit_assignment(left, right),
            Statement::Const(left, _) => {
                panic!("Linter should have removed Const statements {:?}", left)
            }
            Statement::SubCall(b, e) => self.visit_sub_call(b, e),
            Statement::BuiltInSubCall(b, e) => self.visit_built_in_sub_call(b, e),
            Statement::IfBlock(i) => self.visit_if_block(i),
            Statement::SelectCase(s) => self.visit_select_case(s),
            Statement::ForLoop(f) => self.visit_for_loop(f),
            Statement::While(w) => self.visit_conditional_block(w),
            Statement::DoLoop(do_loop_node) => self.visit_do_loop(do_loop_node),
            Statement::OnError(on_error_option) => self.visit_on_error(on_error_option),
            Statement::Label(label) => self.visit_label(label),
            Statement::GoTo(label) => self.visit_go_to(label),
            Statement::Comment(c) => self.visit_comment(c),
            Statement::Dim(dim_list) | Statement::Redim(dim_list) => self.visit_dim(dim_list),
            Statement::Print(print_node) => self.visit_print_node(print_node),
            Statement::GoSub(label) => self.visit_go_sub(label),
            Statement::Resume(resume_option) => self.visit_resume(resume_option),
            Statement::Return(opt_label) => self.visit_return(opt_label.as_ref()),
            Statement::Exit(exit_object) => self.visit_exit(*exit_object),
            Statement::End | Statement::System => Ok(()),
        }
    }

    fn visit_comment(&mut self, _comment: &str) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_dim(&mut self, _dim_list: &DimList) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_on_error(&mut self, _on_error_option: &OnErrorOption) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_label(&mut self, _label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_go_to(&mut self, _label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_go_sub(&mut self, _label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_resume(&mut self, _resume_option: &ResumeOption) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_return(&mut self, _label: Option<&CaseInsensitiveString>) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_exit(&mut self, _exit_object: ExitObject) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_sub_call(
        &mut self,
        _name: &CaseInsensitiveString,
        args: &ExpressionNodes,
    ) -> Result<(), QErrorNode> {
        self.visit_expressions(args)
    }

    fn visit_built_in_sub_call(
        &mut self,
        _name: &BuiltInSub,
        args: &ExpressionNodes,
    ) -> Result<(), QErrorNode> {
        self.visit_expressions(args)
    }

    fn visit_assignment(
        &mut self,
        _name: &Expression,
        v: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        self.visit_expression(v)
    }

    fn visit_for_loop(&mut self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        self.visit_expression(&f.lower_bound)?;
        self.visit_expression(&f.upper_bound)?;
        match &f.step {
            Some(s) => self.visit_expression(s)?,
            None => (),
        }
        self.visit_statement_nodes(&f.statements)
    }

    fn visit_if_block(&mut self, i: &IfBlockNode) -> Result<(), QErrorNode> {
        self.visit_conditional_block(&i.if_block)?;
        for else_if_block in i.else_if_blocks.iter() {
            self.visit_conditional_block(else_if_block)?;
        }
        match &i.else_block {
            Some(x) => self.visit_statement_nodes(x),
            None => Ok(()),
        }
    }

    fn visit_select_case(&mut self, s: &SelectCaseNode) -> Result<(), QErrorNode> {
        self.visit_expression(&s.expr)?;
        for case_block_node in s.case_blocks.iter() {
            self.visit_case_block(case_block_node, &s.expr)?;
        }
        match &s.else_block {
            Some(x) => self.visit_statement_nodes(x),
            None => Ok(()),
        }
    }

    fn visit_case_block(
        &mut self,
        case_block_node: &CaseBlockNode,
        select_expr: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        for case_expr in &case_block_node.expression_list {
            self.visit_case_expression(case_expr, select_expr)?;
        }
        self.visit_statement_nodes(&case_block_node.statements)
    }

    fn visit_case_expression(
        &mut self,
        case_expr: &CaseExpression,
        _select_expr: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        match case_expr {
            CaseExpression::Simple(e) => self.visit_expression(e),
            CaseExpression::Is(_, e) => self.visit_expression(e),
            CaseExpression::Range(from, to) => {
                self.visit_expression(from)?;
                self.visit_expression(to)
            }
        }
    }

    fn visit_conditional_block(&mut self, c: &ConditionalBlockNode) -> Result<(), QErrorNode> {
        self.visit_expression(&c.condition)?;
        self.visit_statement_nodes(&c.statements)
    }

    fn visit_do_loop(&mut self, do_loop_node: &DoLoopNode) -> Result<(), QErrorNode> {
        self.visit_expression(&do_loop_node.condition)?;
        self.visit_statement_nodes(&do_loop_node.statements)
    }

    fn visit_expression(&mut self, _e: &ExpressionNode) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_expressions(&mut self, args: &ExpressionNodes) -> Result<(), QErrorNode> {
        args.iter().try_for_each(|e| self.visit_expression(e))
    }

    fn visit_print_node(&mut self, print_node: &PrintNode) -> Result<(), QErrorNode> {
        match &print_node.format_string {
            Some(f) => self.visit_expression(f)?,
            None => {}
        };
        for print_arg in &print_node.args {
            if let PrintArg::Expression(e) = print_arg {
                self.visit_expression(e)?;
            }
        }
        Ok(())
    }
}
