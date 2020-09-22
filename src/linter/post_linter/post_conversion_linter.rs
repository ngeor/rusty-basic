use crate::built_ins::BuiltInSub;
use crate::common::*;
use crate::linter::types::*;
use crate::parser::QualifiedNameNode;
use crate::variant::Variant;

/// Invoked after the conversion to fully typed program.
/// The default implementation of the trait simply visits all program elements.
///
/// The PostConversionLinter does not modify the elements it visits, which is
/// why it works with references.
///
/// Methods return Ok(()) to indicate an element passes the check or
/// Err() to indicate a problem.
pub trait PostConversionLinter {
    fn visit_program(&self, p: &ProgramNode) -> Result<(), QErrorNode> {
        p.iter()
            .map(|t| self.visit_top_level_token_node(t))
            .collect()
    }

    fn visit_top_level_token_node(&self, t: &TopLevelTokenNode) -> Result<(), QErrorNode> {
        self.visit_top_level_token(t.as_ref()).patch_err_pos(t)
    }

    fn visit_top_level_token(&self, t: &TopLevelToken) -> Result<(), QErrorNode> {
        match t {
            TopLevelToken::FunctionImplementation(f) => self.visit_function_implementation(f),
            TopLevelToken::SubImplementation(s) => self.visit_sub_implementation(s),
            TopLevelToken::Statement(s) => self.visit_statement(s),
        }
    }

    fn visit_function_implementation(&self, f: &FunctionImplementation) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&f.body)
    }

    fn visit_sub_implementation(&self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&s.body)
    }

    fn visit_statement_nodes(&self, s: &StatementNodes) -> Result<(), QErrorNode> {
        s.iter().map(|x| self.visit_statement_node(x)).collect()
    }

    fn visit_statement_node(&self, t: &StatementNode) -> Result<(), QErrorNode> {
        self.visit_statement(t.as_ref()).patch_err_pos(t)
    }

    fn visit_statement(&self, s: &Statement) -> Result<(), QErrorNode> {
        match s {
            Statement::Assignment(left, right) => self.visit_assignment(left, right),
            Statement::Const(left, right) => self.visit_const(left, right),
            Statement::SubCall(b, e) => self.visit_sub_call(b, e),
            Statement::BuiltInSubCall(b, e) => self.visit_built_in_sub_call(b, e),
            Statement::IfBlock(i) => self.visit_if_block(i),
            Statement::SelectCase(s) => self.visit_select_case(s),
            Statement::ForLoop(f) => self.visit_for_loop(f),
            Statement::While(w) => self.visit_conditional_block(w),
            Statement::ErrorHandler(label) => self.visit_error_handler(label),
            Statement::Label(label) => self.visit_label(label),
            Statement::GoTo(label) => self.visit_go_to(label),
            Statement::Comment(c) => self.visit_comment(c),
            Statement::Dim(d) => self.visit_dim(d),
        }
    }

    fn visit_comment(&self, _comment: &String) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_dim(&self, _d: &ResolvedDeclaredNameNode) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_error_handler(&self, _label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_label(&self, _label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_go_to(&self, _label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_sub_call(
        &self,
        _name: &CaseInsensitiveString,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), QErrorNode> {
        self.visit_expressions(args)
    }

    fn visit_built_in_sub_call(
        &self,
        _name: &BuiltInSub,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), QErrorNode> {
        self.visit_expressions(args)
    }

    fn visit_assignment(
        &self,
        _name: &ResolvedDeclaredName,
        v: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        self.visit_expression(v)
    }

    fn visit_for_loop(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        // TODO visit variable name
        self.visit_expression(&f.lower_bound)?;
        self.visit_expression(&f.upper_bound)?;
        match &f.step {
            Some(s) => self.visit_expression(s)?,
            None => (),
        }
        self.visit_statement_nodes(&f.statements)
    }

    fn visit_if_block(&self, i: &IfBlockNode) -> Result<(), QErrorNode> {
        self.visit_conditional_block(&i.if_block)?;
        for else_if_block in i.else_if_blocks.iter() {
            self.visit_conditional_block(else_if_block)?;
        }
        match &i.else_block {
            Some(x) => self.visit_statement_nodes(x),
            None => Ok(()),
        }
    }

    fn visit_select_case(&self, s: &SelectCaseNode) -> Result<(), QErrorNode> {
        self.visit_expression(&s.expr)?;
        for c in s.case_blocks.iter() {
            match &c.expr {
                CaseExpression::Simple(e) => {
                    self.visit_expression(e)?;
                }
                CaseExpression::Is(_, e) => {
                    self.visit_expression(e)?;
                }
                CaseExpression::Range(from, to) => {
                    self.visit_expression(from)?;
                    self.visit_expression(to)?;
                }
            }
            self.visit_statement_nodes(&c.statements)?;
        }
        match &s.else_block {
            Some(x) => self.visit_statement_nodes(x),
            None => Ok(()),
        }
    }

    fn visit_conditional_block(&self, c: &ConditionalBlockNode) -> Result<(), QErrorNode> {
        self.visit_expression(&c.condition)?;
        self.visit_statement_nodes(&c.statements)
    }

    fn visit_const(&self, _left: &QualifiedNameNode, _right: &Variant) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_expression(&self, _e: &ExpressionNode) -> Result<(), QErrorNode> {
        Ok(())
    }

    fn visit_expressions(&self, args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.iter().map(|e| self.visit_expression(e)).collect()
    }
}
