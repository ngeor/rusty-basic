use crate::core::LintErrorPos;
use rusty_common::*;
use rusty_parser::BuiltInSub;
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
    fn visit_program(&mut self, p: &Program) -> Result<(), LintErrorPos> {
        // in case of overriding visit_program, use visit_global_statements to call the default functionality
        self.visit_global_statements(p)
    }

    fn visit_global_statements(&mut self, p: &Program) -> Result<(), LintErrorPos> {
        p.iter()
            .try_for_each(|t| self.visit_global_statement_pos(t))
    }

    fn visit_global_statement_pos(
        &mut self,
        global_statement_pos: &GlobalStatementPos,
    ) -> Result<(), LintErrorPos> {
        let Positioned {
            element: global_statement,
            pos,
        } = global_statement_pos;
        match global_statement {
            GlobalStatement::FunctionImplementation(f) => self.visit_function_implementation(f),
            GlobalStatement::SubImplementation(s) => self.visit_sub_implementation(s),
            GlobalStatement::Statement(s) => self.visit_statement_pos(s, *pos),
            _ => Ok(()),
        }
    }

    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), LintErrorPos> {
        self.visit_statements(&f.body)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), LintErrorPos> {
        self.visit_statements(&s.body)
    }

    fn visit_statements(&mut self, s: &Statements) -> Result<(), LintErrorPos> {
        s.iter()
            .try_for_each(|Positioned { element, pos }| self.visit_statement_pos(element, *pos))
    }

    fn visit_statement_pos(&mut self, s: &Statement, pos: Position) -> Result<(), LintErrorPos> {
        match s {
            Statement::Assignment(a) => self.visit_assignment(a, pos),
            Statement::SubCall(sub_call) => self.visit_sub_call(sub_call, pos),
            Statement::BuiltInSubCall(b, e) => self.visit_built_in_sub_call(b, pos, e),
            Statement::IfBlock(i) => self.visit_if_block(i),
            Statement::SelectCase(s) => self.visit_select_case(s),
            Statement::ForLoop(f) => self.visit_for_loop(f),
            Statement::While(w) => self.visit_conditional_block(w),
            Statement::DoLoop(do_loop) => self.visit_do_loop(do_loop),
            Statement::OnError(on_error_option) => self.visit_on_error(on_error_option, pos),
            Statement::Label(label) => self.visit_label(label, pos),
            Statement::GoTo(label) => self.visit_go_to(label, pos),
            Statement::Comment(c) => self.visit_comment(c),
            Statement::Dim(dim_list) | Statement::Redim(dim_list) => self.visit_dim(dim_list),
            Statement::Print(print) => self.visit_print(print),
            Statement::GoSub(label) => self.visit_go_sub(label, pos),
            Statement::Resume(resume_option) => self.visit_resume(resume_option, pos),
            Statement::Return(opt_label) => self.visit_return(opt_label.as_ref(), pos),
            Statement::Exit(exit_object) => self.visit_exit(*exit_object),
            Statement::Const(_) | Statement::End | Statement::System => Ok(()),
        }
    }

    fn visit_comment(&mut self, _comment: &str) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_dim(&mut self, _dim_list: &DimList) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_on_error(
        &mut self,
        _on_error_option: &OnErrorOption,
        _pos: Position,
    ) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_label(
        &mut self,
        _label: &CaseInsensitiveString,
        _pos: Position,
    ) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_go_to(
        &mut self,
        _label: &CaseInsensitiveString,
        _pos: Position,
    ) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_go_sub(
        &mut self,
        _label: &CaseInsensitiveString,
        _pos: Position,
    ) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_resume(
        &mut self,
        _resume_option: &ResumeOption,
        _pos: Position,
    ) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_return(
        &mut self,
        _label: Option<&CaseInsensitiveString>,
        _pos: Position,
    ) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_exit(&mut self, _exit_object: ExitObject) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_sub_call(&mut self, sub_call: &SubCall, _pos: Position) -> Result<(), LintErrorPos> {
        let (_, args) = sub_call.into();
        self.visit_expressions(args)
    }

    fn visit_built_in_sub_call(
        &mut self,
        _name: &BuiltInSub,
        _pos: Position,
        args: &Expressions,
    ) -> Result<(), LintErrorPos> {
        self.visit_expressions(args)
    }

    fn visit_assignment(
        &mut self,
        assignment: &Assignment,
        _name_pos: Position,
    ) -> Result<(), LintErrorPos> {
        let (_, v) = assignment.into();
        self.visit_expression(v)
    }

    fn visit_for_loop(&mut self, f: &ForLoop) -> Result<(), LintErrorPos> {
        self.visit_expression(&f.lower_bound)?;
        self.visit_expression(&f.upper_bound)?;
        match &f.step {
            Some(s) => self.visit_expression(s)?,
            None => (),
        }
        self.visit_statements(&f.statements)
    }

    fn visit_if_block(&mut self, i: &IfBlock) -> Result<(), LintErrorPos> {
        self.visit_conditional_block(&i.if_block)?;
        for else_if_block in i.else_if_blocks.iter() {
            self.visit_conditional_block(else_if_block)?;
        }
        match &i.else_block {
            Some(x) => self.visit_statements(x),
            None => Ok(()),
        }
    }

    fn visit_select_case(&mut self, s: &SelectCase) -> Result<(), LintErrorPos> {
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
    ) -> Result<(), LintErrorPos> {
        for case_expr in &case_block.expression_list {
            self.visit_case_expression(case_expr, select_expr)?;
        }
        self.visit_statements(&case_block.statements)
    }

    fn visit_case_expression(
        &mut self,
        case_expr: &CaseExpression,
        _select_expr: &ExpressionPos,
    ) -> Result<(), LintErrorPos> {
        match case_expr {
            CaseExpression::Simple(e) => self.visit_expression(e),
            CaseExpression::Is(_, e) => self.visit_expression(e),
            CaseExpression::Range(from, to) => {
                self.visit_expression(from)?;
                self.visit_expression(to)
            }
        }
    }

    fn visit_conditional_block(&mut self, c: &ConditionalBlock) -> Result<(), LintErrorPos> {
        self.visit_expression(&c.condition)?;
        self.visit_statements(&c.statements)
    }

    fn visit_do_loop(&mut self, do_loop: &DoLoop) -> Result<(), LintErrorPos> {
        self.visit_expression(&do_loop.condition)?;
        self.visit_statements(&do_loop.statements)
    }

    fn visit_expression(&mut self, _e: &ExpressionPos) -> Result<(), LintErrorPos> {
        Ok(())
    }

    fn visit_expressions(&mut self, args: &Expressions) -> Result<(), LintErrorPos> {
        args.iter().try_for_each(|e| self.visit_expression(e))
    }

    fn visit_print(&mut self, print: &Print) -> Result<(), LintErrorPos> {
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
