// Convert assignment to set return value (needs resolver)
// No function in const
// For - Next match (needs resolver)

// Stage 1 : convert program node into (statements, subprograms)
// all subs known
// all functions known

// Mission: remove the need for TypeResolver in Interpreter

use super::error::*;
use super::expression_reducer::ExpressionReducer;
use super::post_conversion_linter::PostConversionLinter;
use super::subprogram_context::{collect_subprograms, FunctionMap, SubMap};
use super::types::*;
use crate::common::*;
use crate::parser;
use crate::parser::type_resolver_impl::TypeResolverImpl;
use crate::parser::{
    HasQualifier, Name, NameTrait, Operand, QualifiedName, TypeQualifier, TypeResolver,
};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

//
// Converter trait
//

trait Converter<A, B> {
    fn convert(&mut self, a: A) -> Result<B, Error>;
}

// blanket for Vec
impl<T, A, B> Converter<Vec<A>, Vec<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Vec<A>) -> Result<Vec<B>, Error> {
        a.into_iter().map(|x| self.convert(x)).collect()
    }
}

// blanket for Option
impl<T, A, B> Converter<Option<A>, Option<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Option<A>) -> Result<Option<B>, Error> {
        match a {
            Some(x) => self.convert(x).map(|r| Some(r)),
            None => Ok(None),
        }
    }
}

// blanket for Locatable
impl<T, A, B> Converter<Locatable<A>, Locatable<B>> for T
where
    A: std::fmt::Debug + Sized,
    B: std::fmt::Debug + Sized,
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Locatable<A>) -> Result<Locatable<B>, Error> {
        let (element, pos) = a.consume();
        self.convert(element).with_pos(pos).with_err_pos(pos)
    }
}

//
// Linter
//

#[derive(Debug, Default)]
struct VariableSet(HashMap<CaseInsensitiveString, HashSet<TypeQualifier>>);

impl VariableSet {
    pub fn insert(&mut self, name: QualifiedName) {
        let (bare_name, qualifier) = name.consume();
        match self.0.get_mut(&bare_name) {
            Some(inner_set) => {
                inner_set.insert(qualifier);
            }
            None => {
                let mut inner_set: HashSet<TypeQualifier> = HashSet::new();
                inner_set.insert(qualifier);
                self.0.insert(bare_name, inner_set);
            }
        }
    }

    pub fn contains_qualified(&self, name: &QualifiedName) -> bool {
        match self.0.get(name.bare_name()) {
            Some(inner_set) => inner_set.contains(&name.qualifier()),
            None => false,
        }
    }

    pub fn contains_bare<U: NameTrait>(&self, name: &U) -> bool {
        self.0.contains_key(name.bare_name())
    }
}

#[derive(Debug, Default)]
struct LinterContext {
    parent: Option<Box<LinterContext>>,
    constants: HashMap<CaseInsensitiveString, TypeQualifier>,
    variables: VariableSet,
    function_name: Option<CaseInsensitiveString>,
    sub_name: Option<CaseInsensitiveString>,
}

impl LinterContext {
    pub fn get_constant_type(&self, n: &parser::Name) -> Result<Option<TypeQualifier>, Error> {
        let bare_name: &CaseInsensitiveString = n.bare_name();
        match self.constants.get(bare_name) {
            Some(const_type) => {
                // it's okay to reference a const unqualified
                if n.bare_or_eq(*const_type) {
                    Ok(Some(*const_type))
                } else {
                    Err(LinterError::DuplicateDefinition.into())
                }
            }
            None => Ok(None),
        }
    }

    pub fn get_parent_constant_type(
        &self,
        n: &parser::Name,
    ) -> Result<Option<TypeQualifier>, Error> {
        match &self.parent {
            Some(p) => {
                let x = p.get_constant_type(n)?;
                match x {
                    Some(q) => Ok(Some(q)),
                    None => p.get_parent_constant_type(n),
                }
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Default)]
struct Linter {
    resolver: TypeResolverImpl,
    context: LinterContext,
    functions: FunctionMap,
    subs: SubMap,
}

impl Linter {
    pub fn push_function_context(&mut self, name: &CaseInsensitiveString) {
        let old = std::mem::take(&mut self.context);
        let mut new = LinterContext::default();
        new.parent = Some(Box::new(old));
        new.function_name = Some(name.clone());
        self.context = new;
    }

    pub fn push_sub_context(&mut self, name: &CaseInsensitiveString) {
        let old = std::mem::take(&mut self.context);
        let mut new = LinterContext::default();
        new.parent = Some(Box::new(old));
        new.sub_name = Some(name.clone());
        self.context = new;
    }

    pub fn pop_context(&mut self) {
        let old = std::mem::take(&mut self.context);
        match old.parent {
            Some(p) => {
                self.context = *p;
            }
            None => panic!("Stack underflow!"),
        }
    }
}

pub fn lint(program: parser::ProgramNode) -> Result<ProgramNode, Error> {
    let mut linter = Linter::default();
    let (f_c, s_c) = collect_subprograms(&program)?;
    linter.functions = f_c;
    linter.subs = s_c;
    linter.convert(program)
}

impl Converter<parser::ProgramNode, ProgramNode> for Linter {
    fn convert(&mut self, a: parser::ProgramNode) -> Result<ProgramNode, Error> {
        let mut result: Vec<TopLevelTokenNode> = vec![];
        for top_level_token_node in a.into_iter() {
            // will contain None where DefInt and declarations used to be
            let (top_level_token, pos) = top_level_token_node.consume();
            let opt: Option<TopLevelToken> = self.convert(top_level_token).with_err_pos(pos)?;
            match opt {
                Some(t) => {
                    let r: TopLevelTokenNode = t.at(pos);
                    result.push(r);
                }
                _ => (),
            }
        }

        let linter = super::no_dynamic_const::NoDynamicConst {};
        linter.visit_program(&result)?;

        let linter = super::for_next_counter_match::ForNextCounterMatch {};
        linter.visit_program(&result)?;

        let linter = super::built_in_function_linter::BuiltInFunctionLinter {};
        linter.visit_program(&result)?;

        let linter = super::built_in_sub_linter::BuiltInSubLinter {};
        linter.visit_program(&result)?;

        let linter = super::user_defined_function_linter::UserDefinedFunctionLinter {
            functions: &self.functions,
        };
        linter.visit_program(&result)?;

        let linter = super::user_defined_sub_linter::UserDefinedSubLinter { subs: &self.subs };
        linter.visit_program(&result)?;

        let linter = super::select_case_linter::SelectCaseLinter {};
        linter.visit_program(&result)?;

        let mut linter = super::label_linter::LabelLinter::new();
        linter.visit_program(&result)?;
        linter.switch_to_validating_mode();
        linter.visit_program(&result)?;

        let reducer = super::undefined_function_reducer::UndefinedFunctionReducer {
            functions: &self.functions,
        };
        result = reducer.visit_program(result)?;

        Ok(result)
    }
}

impl Converter<Name, QualifiedName> for Linter {
    fn convert(&mut self, a: Name) -> Result<QualifiedName, Error> {
        match a {
            Name::Bare(b) => {
                let qualifier = self.resolver.resolve(&b);
                Ok(QualifiedName::new(b, qualifier))
            }
            Name::Qualified(q) => Ok(q),
        }
    }
}

// Option because we filter out DefType
impl Converter<parser::TopLevelToken, Option<TopLevelToken>> for Linter {
    fn convert(&mut self, a: parser::TopLevelToken) -> Result<Option<TopLevelToken>, Error> {
        match a {
            parser::TopLevelToken::DefType(d) => {
                self.resolver.set(&d);
                Ok(None)
            }
            parser::TopLevelToken::FunctionDeclaration(_, _)
            | parser::TopLevelToken::SubDeclaration(_, _) => Ok(None),
            parser::TopLevelToken::FunctionImplementation(n, params, block) => {
                let mapped_name = self.convert(n)?;
                let mapped_params = self.convert(params)?;
                self.push_function_context(mapped_name.bare_name());
                for q_n_n in mapped_params.iter() {
                    self.context.variables.insert(q_n_n.as_ref().clone());
                }
                let mapped = TopLevelToken::FunctionImplementation(FunctionImplementation {
                    name: mapped_name,
                    params: mapped_params,
                    body: self.convert(block)?,
                });
                self.pop_context();
                Ok(Some(mapped))
            }
            parser::TopLevelToken::SubImplementation(n, params, block) => {
                let mapped_params = self.convert(params)?;
                self.push_sub_context(n.bare_name());
                for q_n_n in mapped_params.iter() {
                    self.context.variables.insert(q_n_n.as_ref().clone());
                }
                let mapped = TopLevelToken::SubImplementation(SubImplementation {
                    name: n,
                    params: mapped_params,
                    body: self.convert(block)?,
                });
                self.pop_context();
                Ok(Some(mapped))
            }
            parser::TopLevelToken::Statement(s) => {
                Ok(Some(TopLevelToken::Statement(self.convert(s)?)))
            }
        }
    }
}

impl Converter<parser::Statement, Statement> for Linter {
    fn convert(&mut self, a: parser::Statement) -> Result<Statement, Error> {
        match a {
            parser::Statement::Assignment(n, e) => {
                if self
                    .context
                    .function_name
                    .as_ref()
                    .map(|x| x == n.bare_name())
                    .unwrap_or_default()
                {
                    // trying to assign to the function
                    let function_type: TypeQualifier = self.functions.get(n.bare_name()).unwrap().0;
                    if n.bare_or_eq(function_type) {
                        let converted_expr: ExpressionNode = self.convert(e)?;
                        let result_q: TypeQualifier = converted_expr.as_ref().try_qualifier()?;
                        if result_q.can_cast_to(function_type) {
                            Ok(Statement::SetReturnValue(converted_expr))
                        } else {
                            err_l(LinterError::TypeMismatch, &converted_expr)
                        }
                    } else {
                        // trying to assign to the function with an explicit wrong type
                        Err(LinterError::DuplicateDefinition.into())
                    }
                } else if self
                    .context
                    .sub_name
                    .as_ref()
                    .map(|x| x == n.bare_name())
                    .unwrap_or_default()
                {
                    // trying to assign to the sub name should always be an error hopefully
                    Err(LinterError::InvalidAssignment.into())
                } else {
                    if self.context.constants.contains_key(n.bare_name()) {
                        // cannot overwrite local constant
                        Err(LinterError::DuplicateDefinition.into())
                    } else {
                        let converted_name = self.convert(n)?;
                        let converted_expr: ExpressionNode = self.convert(e)?;
                        let result_q: TypeQualifier = converted_expr.as_ref().try_qualifier()?;
                        if result_q.can_cast_to(converted_name.qualifier()) {
                            self.context.variables.insert(converted_name.clone());
                            Ok(Statement::Assignment(converted_name, converted_expr))
                        } else {
                            err_l(LinterError::TypeMismatch, &converted_expr)
                        }
                    }
                }
            }
            parser::Statement::Const(n, e) => {
                let (name, pos) = n.consume();
                if self.context.variables.contains_bare(&name)
                    || self.context.constants.contains_key(name.bare_name())
                {
                    // local variable or local constant already present by that name
                    err(LinterError::DuplicateDefinition, pos)
                } else {
                    let converted_expression_node = self.convert(e)?;
                    let e_type = converted_expression_node.as_ref().try_qualifier()?;
                    match name {
                        Name::Bare(b) => {
                            // bare name resolves from right side, not resolver
                            self.context.constants.insert(b.clone(), e_type);
                            Ok(Statement::Const(
                                QualifiedName::new(b, e_type).at(pos),
                                converted_expression_node,
                            ))
                        }
                        Name::Qualified(q) => {
                            if e_type.can_cast_to(q.qualifier()) {
                                self.context
                                    .constants
                                    .insert(q.bare_name().clone(), q.qualifier());
                                Ok(Statement::Const(q.at(pos), converted_expression_node))
                            } else {
                                err_l(LinterError::TypeMismatch, &converted_expression_node)
                            }
                        }
                    }
                }
            }
            parser::Statement::SubCall(n, args) => {
                let converted_args = self.convert(args)?;
                let opt_built_in: Option<BuiltInSub> = (&n).into();
                match opt_built_in {
                    Some(b) => Ok(Statement::BuiltInSubCall(b, converted_args)),
                    None => Ok(Statement::SubCall(n, converted_args)),
                }
            }
            parser::Statement::IfBlock(i) => Ok(Statement::IfBlock(self.convert(i)?)),
            parser::Statement::SelectCase(s) => Ok(Statement::SelectCase(self.convert(s)?)),
            parser::Statement::ForLoop(f) => Ok(Statement::ForLoop(self.convert(f)?)),
            parser::Statement::While(c) => Ok(Statement::While(self.convert(c)?)),
            parser::Statement::ErrorHandler(l) => Ok(Statement::ErrorHandler(l)),
            parser::Statement::Label(l) => Ok(Statement::Label(l)),
            parser::Statement::GoTo(l) => Ok(Statement::GoTo(l)),
        }
    }
}

impl Converter<parser::Expression, Expression> for Linter {
    fn convert(&mut self, a: parser::Expression) -> Result<Expression, Error> {
        match a {
            parser::Expression::SingleLiteral(f) => Ok(Expression::SingleLiteral(f)),
            parser::Expression::DoubleLiteral(f) => Ok(Expression::DoubleLiteral(f)),
            parser::Expression::StringLiteral(f) => Ok(Expression::StringLiteral(f)),
            parser::Expression::IntegerLiteral(f) => Ok(Expression::IntegerLiteral(f)),
            parser::Expression::LongLiteral(f) => Ok(Expression::LongLiteral(f)),
            parser::Expression::VariableName(n) => {
                // check for a local constant
                match self.context.get_constant_type(&n)? {
                    Some(q) => Ok(Expression::Constant(QualifiedName::new(
                        n.bare_name().clone(),
                        q,
                    ))),
                    None => {
                        // check for an already defined local variable or parameter
                        let converted_name = self.convert(n.clone())?;
                        if self.context.variables.contains_qualified(&converted_name) {
                            Ok(Expression::Variable(converted_name))
                        } else {
                            // parent constant?
                            match self.context.get_parent_constant_type(&n)? {
                                Some(q) => Ok(Expression::Constant(QualifiedName::new(
                                    n.bare_name().clone(),
                                    q,
                                ))),
                                None => {
                                    // e.g. INPUT N, where N has not been declared in advance
                                    self.context.variables.insert(converted_name.clone());
                                    Ok(Expression::Variable(converted_name))
                                }
                            }
                        }
                    }
                }
            }
            parser::Expression::FunctionCall(n, args) => {
                let converted_args = self.convert(args)?;
                let opt_built_in: Option<BuiltInFunction> = (&n).try_into()?;
                match opt_built_in {
                    Some(b) => Ok(Expression::BuiltInFunctionCall(b, converted_args)),
                    None => Ok(Expression::FunctionCall(self.convert(n)?, converted_args)),
                }
            }
            parser::Expression::BinaryExpression(op, l, r) => {
                // unbox them
                let unboxed_left = *l;
                let unboxed_right = *r;
                // convert them
                let converted_left = self.convert(unboxed_left)?;
                let converted_right = self.convert(unboxed_right)?;
                // get the types
                let q_left = converted_left.as_ref().try_qualifier()?;
                let q_right = converted_right.as_ref().try_qualifier()?;
                // can we cast from right to left?
                let can_cast = q_right.can_cast_to(q_left);
                // plus extra checks
                let is_valid_op = match op {
                    // you can't do "A" - "B"
                    Operand::Minus => can_cast && q_left != TypeQualifier::DollarString,
                    _ => can_cast,
                };
                if is_valid_op {
                    Ok(Expression::BinaryExpression(
                        op,
                        Box::new(converted_left),
                        Box::new(converted_right),
                    ))
                } else {
                    err_l(LinterError::TypeMismatch, &converted_right)
                }
            }
            parser::Expression::UnaryExpression(op, c) => {
                let unboxed_child = *c;
                let converted_child = self.convert(unboxed_child)?;
                let converted_q = converted_child.as_ref().try_qualifier()?;
                if converted_q == TypeQualifier::DollarString {
                    // no unary operation works for strings
                    err_l(LinterError::TypeMismatch, &converted_child)
                } else {
                    Ok(Expression::UnaryExpression(op, Box::new(converted_child)))
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

impl Converter<parser::ForLoopNode, ForLoopNode> for Linter {
    fn convert(&mut self, a: parser::ForLoopNode) -> Result<ForLoopNode, Error> {
        Ok(ForLoopNode {
            variable_name: self.convert(a.variable_name)?,
            lower_bound: self.convert(a.lower_bound)?,
            upper_bound: self.convert(a.upper_bound)?,
            step: self.convert(a.step)?,
            statements: self.convert(a.statements)?,
            next_counter: self.convert(a.next_counter)?,
        })
    }
}

impl Converter<parser::ConditionalBlockNode, ConditionalBlockNode> for Linter {
    fn convert(&mut self, a: parser::ConditionalBlockNode) -> Result<ConditionalBlockNode, Error> {
        Ok(ConditionalBlockNode {
            condition: self.convert(a.condition)?,
            statements: self.convert(a.statements)?,
        })
    }
}

impl Converter<parser::IfBlockNode, IfBlockNode> for Linter {
    fn convert(&mut self, a: parser::IfBlockNode) -> Result<IfBlockNode, Error> {
        Ok(IfBlockNode {
            if_block: self.convert(a.if_block)?,
            else_if_blocks: self.convert(a.else_if_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}

impl Converter<parser::SelectCaseNode, SelectCaseNode> for Linter {
    fn convert(&mut self, a: parser::SelectCaseNode) -> Result<SelectCaseNode, Error> {
        Ok(SelectCaseNode {
            expr: self.convert(a.expr)?,
            case_blocks: self.convert(a.case_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}

impl Converter<parser::CaseBlockNode, CaseBlockNode> for Linter {
    fn convert(&mut self, a: parser::CaseBlockNode) -> Result<CaseBlockNode, Error> {
        Ok(CaseBlockNode {
            expr: self.convert(a.expr)?,
            statements: self.convert(a.statements)?,
        })
    }
}

impl Converter<parser::CaseExpression, CaseExpression> for Linter {
    fn convert(&mut self, a: parser::CaseExpression) -> Result<CaseExpression, Error> {
        match a {
            parser::CaseExpression::Simple(e) => self.convert(e).map(|x| CaseExpression::Simple(x)),
            parser::CaseExpression::Is(op, e) => self.convert(e).map(|x| CaseExpression::Is(op, x)),
            parser::CaseExpression::Range(from, to) => self
                .convert(from)
                .and_then(|x| self.convert(to).map(|y| CaseExpression::Range(x, y))),
        }
    }
}
