use super::error::*;
use super::subprogram_context::{collect_subprograms, FunctionMap, SubMap};
use super::types::*;
use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::linter::linter_context::LinterContext;
use crate::parser;
use crate::parser::type_resolver_impl::TypeResolverImpl;
use crate::parser::{
    DimType, HasQualifier, Name, NameTrait, QualifiedName, TypeQualifier, TypeResolver,
};
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
// Converter
//

#[derive(Debug, Default)]
struct ConverterImpl {
    resolver: TypeResolverImpl,
    context: LinterContext,
    functions: FunctionMap,
    subs: SubMap,
}

impl ConverterImpl {
    pub fn push_function_context(&mut self, name: &CaseInsensitiveString) {
        let old = std::mem::take(&mut self.context);
        self.context = old.push_function_context(name);
    }

    pub fn push_sub_context(&mut self, name: &CaseInsensitiveString) {
        let old = std::mem::take(&mut self.context);
        self.context = old.push_sub_context(name);
    }

    pub fn pop_context(&mut self) {
        let old = std::mem::take(&mut self.context);
        self.context = old.pop_context();
    }

    pub fn consume(self) -> (FunctionMap, SubMap) {
        (self.functions, self.subs)
    }
}

pub fn convert(program: parser::ProgramNode) -> Result<(ProgramNode, FunctionMap, SubMap), Error> {
    let mut linter = ConverterImpl::default();
    let (f_c, s_c) = collect_subprograms(&program)?;
    linter.functions = f_c;
    linter.subs = s_c;
    let result = linter.convert(program)?;
    let (f, s) = linter.consume();
    Ok((result, f, s))
}

impl Converter<parser::ProgramNode, ProgramNode> for ConverterImpl {
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
        Ok(result)
    }
}

impl Converter<Name, QualifiedName> for ConverterImpl {
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
impl Converter<parser::TopLevelToken, Option<TopLevelToken>> for ConverterImpl {
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
                    if self.functions.contains_key(q_n_n.bare_name())
                        || self.subs.contains_key(q_n_n.bare_name())
                    {
                        // not possible to have a param name that clashes with a sub or function
                        return err_l(LinterError::DuplicateDefinition, q_n_n);
                    }
                    self.context.variables().insert(q_n_n.as_ref().clone());
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
                    self.context.variables().insert(q_n_n.as_ref().clone());
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

impl Converter<parser::Statement, Statement> for ConverterImpl {
    fn convert(&mut self, a: parser::Statement) -> Result<Statement, Error> {
        match a {
            parser::Statement::Comment(c) => Ok(Statement::Comment(c)),
            parser::Statement::Assignment(n, e) => {
                let resolved_l_name = self.resolve_name_in_assignment(n)?;
                let name_type = resolved_l_name.qualifier();
                let converted_expr: ExpressionNode = self.convert(e)?;
                let result_q: TypeQualifier = converted_expr.try_qualifier()?;
                if result_q.can_cast_to(name_type) {
                    match resolved_l_name {
                        LName::Variable(q) => Ok(Statement::Assignment(q, converted_expr)),
                        LName::Function(_) => Ok(Statement::SetReturnValue(converted_expr)),
                    }
                } else {
                    err_l(LinterError::TypeMismatch, &converted_expr)
                }
            }
            parser::Statement::Const(n, e) => {
                let (name, pos) = n.consume();
                if self.context.variables().contains_bare(&name)
                    || self.context.constants().contains_key(name.bare_name())
                    || self.functions.contains_key(name.bare_name())
                    || self.subs.contains_key(name.bare_name())
                    || self.context.dim_variables().contains_key(name.bare_name())
                {
                    // local variable or local constant or function or sub already present by that name
                    err(LinterError::DuplicateDefinition, pos)
                } else {
                    let converted_expression_node = self.convert(e)?;
                    let e_type = converted_expression_node.try_qualifier()?;
                    match name {
                        Name::Bare(b) => {
                            // bare name resolves from right side, not resolver
                            self.context.constants().insert(b.clone(), e_type);
                            Ok(Statement::Const(
                                QualifiedName::new(b, e_type).at(pos),
                                converted_expression_node,
                            ))
                        }
                        Name::Qualified(q) => {
                            if e_type.can_cast_to(q.qualifier()) {
                                self.context
                                    .constants()
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
            parser::Statement::Dim(name, dim_type) => {
                match dim_type {
                    DimType::BuiltInType(b) => {
                        if self.context.dim_variables().contains_key(&name)
                            || self.context.constants().contains_key(&name)
                        {
                            return err_no_pos(LinterError::DuplicateDefinition);
                        } else {
                            self.context.dim_variables().insert(name.clone(), b);
                        }
                    }
                    DimType::UserDefinedType(_) => {
                        unimplemented!();
                    }
                };
                Ok(Statement::Dim(name, dim_type))
            }
        }
    }
}

impl Converter<parser::Expression, Expression> for ConverterImpl {
    fn convert(&mut self, a: parser::Expression) -> Result<Expression, Error> {
        match a {
            parser::Expression::SingleLiteral(f) => Ok(Expression::SingleLiteral(f)),
            parser::Expression::DoubleLiteral(f) => Ok(Expression::DoubleLiteral(f)),
            parser::Expression::StringLiteral(f) => Ok(Expression::StringLiteral(f)),
            parser::Expression::IntegerLiteral(f) => Ok(Expression::IntegerLiteral(f)),
            parser::Expression::LongLiteral(f) => Ok(Expression::LongLiteral(f)),
            parser::Expression::VariableName(n) => self.resolve_name_in_expression(&n),
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
                let q_left = converted_left.try_qualifier()?;
                let q_right = converted_right.try_qualifier()?;
                // get the cast type
                let result_type = super::operand_type::cast_binary_op(op, q_left, q_right);
                if result_type.is_some() {
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
                let converted_q = converted_child.try_qualifier()?;
                if super::operand_type::cast_unary_op(op, converted_q).is_none() {
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
            parser::Expression::FileHandle(i) => Ok(Expression::FileHandle(i)),
        }
    }
}

impl Converter<parser::ForLoopNode, ForLoopNode> for ConverterImpl {
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

impl Converter<parser::ConditionalBlockNode, ConditionalBlockNode> for ConverterImpl {
    fn convert(&mut self, a: parser::ConditionalBlockNode) -> Result<ConditionalBlockNode, Error> {
        Ok(ConditionalBlockNode {
            condition: self.convert(a.condition)?,
            statements: self.convert(a.statements)?,
        })
    }
}

impl Converter<parser::IfBlockNode, IfBlockNode> for ConverterImpl {
    fn convert(&mut self, a: parser::IfBlockNode) -> Result<IfBlockNode, Error> {
        Ok(IfBlockNode {
            if_block: self.convert(a.if_block)?,
            else_if_blocks: self.convert(a.else_if_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}

impl Converter<parser::SelectCaseNode, SelectCaseNode> for ConverterImpl {
    fn convert(&mut self, a: parser::SelectCaseNode) -> Result<SelectCaseNode, Error> {
        Ok(SelectCaseNode {
            expr: self.convert(a.expr)?,
            case_blocks: self.convert(a.case_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}

impl Converter<parser::CaseBlockNode, CaseBlockNode> for ConverterImpl {
    fn convert(&mut self, a: parser::CaseBlockNode) -> Result<CaseBlockNode, Error> {
        Ok(CaseBlockNode {
            expr: self.convert(a.expr)?,
            statements: self.convert(a.statements)?,
        })
    }
}

impl Converter<parser::CaseExpression, CaseExpression> for ConverterImpl {
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

enum LName {
    Variable(QualifiedName),
    Function(QualifiedName),
}

impl HasQualifier for LName {
    fn qualifier(&self) -> TypeQualifier {
        match self {
            Self::Variable(n) | Self::Function(n) => n.qualifier(),
        }
    }
}

impl ConverterImpl {
    pub fn resolve_name_in_assignment(&mut self, n: parser::Name) -> Result<LName, Error> {
        if self.context.is_function_context(&n) {
            // trying to assign to the function
            let function_type: TypeQualifier = self.functions.get(n.bare_name()).unwrap().0;
            if n.bare_or_eq(function_type) {
                Ok(LName::Function(QualifiedName::new(
                    n.bare_name().clone(),
                    function_type,
                )))
            } else {
                // trying to assign to the function with an explicit wrong type
                Err(LinterError::DuplicateDefinition.into())
            }
        } else if self.functions.contains_key(n.bare_name())
            || self.subs.contains_key(n.bare_name())
            || self.context.constants().contains_key(n.bare_name())
        {
            // trying to assign to a different function, or to a sub, or to overwrite a local constant
            Err(LinterError::DuplicateDefinition.into())
        } else if n.is_bare() && self.context.dim_variables().contains_key(n.bare_name()) {
            // intercept assignment to DIM variable e.g.
            // DIM A AS STRING
            // A = "hello"
            let q = *self.context.dim_variables().get(n.bare_name()).unwrap();
            Ok(LName::Variable(QualifiedName::new(
                n.bare_name().clone(),
                q,
            )))
        } else {
            let converted_name = self.convert(n)?;
            self.context.variables().insert(converted_name.clone());
            Ok(LName::Variable(converted_name))
        }
    }

    pub fn resolve_name_in_expression(&mut self, n: &parser::Name) -> Result<Expression, Error> {
        self.resolve_name_as_const(n)
            .or_try_read(|| self.resolve_name_as_existing_var(n))
            .or_try_read(|| self.resolve_name_as_parent_const(n))
            .or_try_read(|| self.resolve_name_as_parent_const(n))
            .or_try_read(|| self.resolve_name_as_subprogram(n))
            .or_try_read(|| self.resolve_name_as_dim(n))
            .or_read(|| self.resolve_name_as_new_var(n))
    }

    fn resolve_name_as_const(&mut self, n: &parser::Name) -> Result<Option<Expression>, Error> {
        match self.context.get_constant_type(n)? {
            Some(q) => Ok(Some(Expression::Constant(QualifiedName::new(
                n.bare_name().clone(),
                q,
            )))),
            None => Ok(None),
        }
    }

    fn resolve_name_as_parent_const(
        &mut self,
        n: &parser::Name,
    ) -> Result<Option<Expression>, Error> {
        match self.context.get_parent_constant_type(n)? {
            Some(q) => Ok(Some(Expression::Constant(QualifiedName::new(
                n.bare_name().clone(),
                q,
            )))),
            None => Ok(None),
        }
    }

    fn resolve_name_as_existing_var(
        &mut self,
        n: &parser::Name,
    ) -> Result<Option<Expression>, Error> {
        let converted_name = self.convert(n.clone())?;
        if self.context.variables().contains_qualified(&converted_name) {
            Ok(Some(Expression::Variable(converted_name)))
        } else {
            Ok(None)
        }
    }

    fn resolve_name_as_new_var(&mut self, n: &parser::Name) -> Result<Expression, Error> {
        // e.g. INPUT N, where N has not been declared in advance
        let converted_name = self.convert(n.clone())?;
        self.context.variables().insert(converted_name.clone());
        Ok(Expression::Variable(converted_name))
    }

    fn resolve_name_as_subprogram(
        &mut self,
        n: &parser::Name,
    ) -> Result<Option<Expression>, Error> {
        if self.subs.contains_key(n.bare_name()) {
            // using the name of a sub as a variable expression
            err_no_pos(LinterError::DuplicateDefinition)
        } else if self.functions.contains_key(n.bare_name()) {
            // if the function expects arguments, argument count mismatch
            let (f_type, f_args, _) = self.functions.get(n.bare_name()).unwrap();
            if !f_args.is_empty() {
                err_no_pos(LinterError::ArgumentCountMismatch)
            } else if !n.bare_or_eq(*f_type) {
                // if the function is a different type and the name is qualified of a different type, duplication definition
                err_no_pos(LinterError::DuplicateDefinition)
            } else {
                // else convert it to function call
                Ok(Some(Expression::FunctionCall(
                    QualifiedName::new(n.bare_name().clone(), *f_type),
                    vec![],
                )))
            }
        } else {
            Ok(None)
        }
    }

    fn resolve_name_as_dim(&mut self, n: &parser::Name) -> Result<Option<Expression>, Error> {
        match n {
            parser::Name::Bare(bare_name) => {
                match self.context.dim_variables().get(bare_name) {
                    Some(q) => {
                        // found it as bare name
                        Ok(Some(Expression::Variable(QualifiedName::new(
                            bare_name, *q,
                        ))))
                    }
                    None => Ok(None),
                }
            }
            parser::Name::Qualified(q_name) => {
                match self.context.dim_variables().get(q_name.bare_name()) {
                    Some(q) => {
                        if *q == q_name.qualifier() {
                            Ok(Some(Expression::Variable(q_name.clone())))
                        } else {
                            err_no_pos(LinterError::DuplicateDefinition)
                        }
                    }
                    None => Ok(None),
                }
            }
        }
    }
}
