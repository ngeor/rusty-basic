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
    fn visit_program(&mut self, p: &Program) -> Result<(), QErrorPos> {
        // in case of overriding visit_program, use visit_global_statements to call the default functionality
        self.visit_global_statements(p)
    }

    fn visit_global_statements(&mut self, p: &Program) -> Result<(), QErrorPos> {
        p.iter()
            .try_for_each(|t| self.visit_global_statement_pos(t))
    }

    fn visit_global_statement_pos(
        &mut self,
        global_statement_pos: &GlobalStatementPos,
    ) -> Result<(), QErrorPos> {
        self.visit_global_statement(&global_statement_pos.element)
            .patch_err_pos(global_statement_pos)
    }

    fn visit_global_statement(
        &mut self,
        global_statement: &GlobalStatement,
    ) -> Result<(), QErrorPos> {
        match global_statement {
            GlobalStatement::FunctionImplementation(f) => self.visit_function_implementation(f),
            GlobalStatement::SubImplementation(s) => self.visit_sub_implementation(s),
            GlobalStatement::Statement(s) => self.visit_statement(s),
            _ => Ok(()),
        }
    }

    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorPos> {
        self.visit_statements(&f.body)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorPos> {
        self.visit_statements(&s.body)
    }

    fn visit_statements(&mut self, s: &Statements) -> Result<(), QErrorPos> {
        s.iter().try_for_each(|x| self.visit_statement_pos(x))
    }

    fn visit_statement_pos(&mut self, t: &StatementPos) -> Result<(), QErrorPos> {
        self.visit_statement(&t.element).patch_err_pos(t)
    }

    fn visit_statement(&mut self, s: &Statement) -> Result<(), QErrorPos> {
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
            Statement::DoLoop(do_loop) => self.visit_do_loop(do_loop),
            Statement::OnError(on_error_option) => self.visit_on_error(on_error_option),
            Statement::Label(label) => self.visit_label(label),
            Statement::GoTo(label) => self.visit_go_to(label),
            Statement::Comment(c) => self.visit_comment(c),
            Statement::Dim(dim_list) | Statement::Redim(dim_list) => self.visit_dim(dim_list),
            Statement::Print(print) => self.visit_print(print),
            Statement::GoSub(label) => self.visit_go_sub(label),
            Statement::Resume(resume_option) => self.visit_resume(resume_option),
            Statement::Return(opt_label) => self.visit_return(opt_label.as_ref()),
            Statement::Exit(exit_object) => self.visit_exit(*exit_object),
            Statement::End | Statement::System => Ok(()),
        }
    }

    fn visit_comment(&mut self, _comment: &str) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_dim(&mut self, _dim_list: &DimList) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_on_error(&mut self, _on_error_option: &OnErrorOption) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_label(&mut self, _label: &CaseInsensitiveString) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_go_to(&mut self, _label: &CaseInsensitiveString) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_go_sub(&mut self, _label: &CaseInsensitiveString) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_resume(&mut self, _resume_option: &ResumeOption) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_return(&mut self, _label: Option<&CaseInsensitiveString>) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_exit(&mut self, _exit_object: ExitObject) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_sub_call(
        &mut self,
        _name: &CaseInsensitiveString,
        args: &Expressions,
    ) -> Result<(), QErrorPos> {
        self.visit_expressions(args)
    }

    fn visit_built_in_sub_call(
        &mut self,
        _name: &BuiltInSub,
        args: &Expressions,
    ) -> Result<(), QErrorPos> {
        self.visit_expressions(args)
    }

    fn visit_assignment(&mut self, _name: &Expression, v: &ExpressionPos) -> Result<(), QErrorPos> {
        self.visit_expression(v)
    }

    fn visit_for_loop(&mut self, f: &ForLoop) -> Result<(), QErrorPos> {
        self.visit_expression(&f.lower_bound)?;
        self.visit_expression(&f.upper_bound)?;
        match &f.step {
            Some(s) => self.visit_expression(s)?,
            None => (),
        }
        self.visit_statements(&f.statements)
    }

    fn visit_if_block(&mut self, i: &IfBlock) -> Result<(), QErrorPos> {
        self.visit_conditional_block(&i.if_block)?;
        for else_if_block in i.else_if_blocks.iter() {
            self.visit_conditional_block(else_if_block)?;
        }
        match &i.else_block {
            Some(x) => self.visit_statements(x),
            None => Ok(()),
        }
    }

    fn visit_select_case(&mut self, s: &SelectCase) -> Result<(), QErrorPos> {
        self.visit_expression(&s.expr)?;
        for case_block in s.case_blocks.iter() {
            self.visit_case_block(case_block, &s.expr)?;
        }
        match &s.else_block {
            Some(x) => self.visit_statements(x),
            None => Ok(()),
        }
    }

    fn visit_case_block(
        &mut self,
        case_block: &CaseBlock,
        select_expr: &ExpressionPos,
    ) -> Result<(), QErrorPos> {
        for case_expr in &case_block.expression_list {
            self.visit_case_expression(case_expr, select_expr)?;
        }
        self.visit_statements(&case_block.statements)
    }

    fn visit_case_expression(
        &mut self,
        case_expr: &CaseExpression,
        _select_expr: &ExpressionPos,
    ) -> Result<(), QErrorPos> {
        match case_expr {
            CaseExpression::Simple(e) => self.visit_expression(e),
            CaseExpression::Is(_, e) => self.visit_expression(e),
            CaseExpression::Range(from, to) => {
                self.visit_expression(from)?;
                self.visit_expression(to)
            }
        }
    }

    fn visit_conditional_block(&mut self, c: &ConditionalBlock) -> Result<(), QErrorPos> {
        self.visit_expression(&c.condition)?;
        self.visit_statements(&c.statements)
    }

    fn visit_do_loop(&mut self, do_loop: &DoLoop) -> Result<(), QErrorPos> {
        self.visit_expression(&do_loop.condition)?;
        self.visit_statements(&do_loop.statements)
    }

    fn visit_expression(&mut self, _e: &ExpressionPos) -> Result<(), QErrorPos> {
        Ok(())
    }

    fn visit_expressions(&mut self, args: &Expressions) -> Result<(), QErrorPos> {
        args.iter().try_for_each(|e| self.visit_expression(e))
    }

    fn visit_print(&mut self, print: &Print) -> Result<(), QErrorPos> {
        match &print.format_string {
            Some(f) => self.visit_expression(f)?,
            None => {}
        };
        for print_arg in &print.args {
            if let PrintArg::Expression(e) = print_arg {
                self.visit_expression(e)?;
            }
        }
        Ok(())
    }
}
