use super::subprogram_context::{FirstPassOuter, FunctionMap, SubMap};
use super::types::*;
use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::linter::linter_context::LinterContext;
use crate::linter::type_resolver::*;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser;
use crate::parser::{
    BareName, BareNameNode, CanCastTo, DeclaredName, DeclaredNameNodes, HasQualifier, Name,
    NameNode, QualifiedName, QualifiedNameNode, TypeDefinition, TypeQualifier, WithTypeQualifier,
};
use std::convert::TryInto;

//
// Converter trait
//

trait Converter<A, B> {
    fn convert(&mut self, a: A) -> Result<B, QErrorNode>;
}

// blanket for Vec
impl<T, A, B> Converter<Vec<A>, Vec<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Vec<A>) -> Result<Vec<B>, QErrorNode> {
        a.into_iter().map(|x| self.convert(x)).collect()
    }
}

// blanket for Option
impl<T, A, B> Converter<Option<A>, Option<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Option<A>) -> Result<Option<B>, QErrorNode> {
        match a {
            Some(x) => self.convert(x).map(|r| Some(r)),
            None => Ok(None),
        }
    }
}

// blanket for Locatable
impl<T, A, B> Converter<Locatable<A>, Locatable<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Locatable<A>) -> Result<Locatable<B>, QErrorNode> {
        let Locatable { element, pos } = a;
        self.convert(element).with_ok_pos(pos).patch_err_pos(pos)
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
    function_declarations: FunctionMap,
    sub_declarations: SubMap,
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

    fn convert_function_implementation(
        &mut self,
        function_name_node: NameNode,
        params: DeclaredNameNodes,
        block: parser::StatementNodes,
    ) -> Result<Option<TopLevelToken>, QErrorNode> {
        let mapped_name = self.convert(function_name_node)?;
        self.push_function_context(mapped_name.as_ref());
        let mapped_params = self.convert_function_params(mapped_name.as_ref(), params)?;
        let mapped = TopLevelToken::FunctionImplementation(FunctionImplementation {
            name: mapped_name,
            params: mapped_params,
            body: self.convert(block)?,
        });
        self.pop_context();
        Ok(Some(mapped))
    }

    fn resolve_declared_name(&self, d: &DeclaredName) -> TypeQualifier {
        match d.type_definition() {
            TypeDefinition::Bare => {
                let bare_name: &BareName = d.as_ref();
                bare_name.resolve_into(&self.resolver)
            }
            TypeDefinition::CompactBuiltIn(q) | TypeDefinition::ExtendedBuiltIn(q) => *q,
            _ => unimplemented!(),
        }
    }

    fn convert_function_params(
        &mut self,
        function_name: &QualifiedName,
        params: DeclaredNameNodes,
    ) -> Result<ResolvedDeclaredNameNodes, QErrorNode> {
        let mut result: ResolvedDeclaredNameNodes = vec![];
        for p in params.into_iter() {
            let Locatable {
                element: declared_name,
                pos,
            } = p;
            let bare_name: &BareName = declared_name.as_ref();
            if self.subs.contains_key(bare_name) {
                // not possible to have a param name that clashes with a sub (functions are ok)
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let q: TypeQualifier = self.resolve_declared_name(&declared_name);
            if function_name.as_ref() == bare_name
                && (function_name.qualifier() != q || declared_name.is_extended())
            {
                // not possible to have a param name clashing with the function name if the type is different or if it's an extended declaration (AS SINGLE)
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            // TODO it is not always CompactBuiltIn, support ExtendedBuiltIn, if it makes a difference
            let resolved_declared_name = ResolvedDeclaredName {
                name: bare_name.clone(),
                type_definition: ResolvedTypeDefinition::CompactBuiltIn(q),
            };
            self.context
                .push_param(declared_name, &self.resolver)
                .with_err_at(pos)?;
            result.push(resolved_declared_name.at(pos));
        }
        Ok(result)
    }

    fn convert_sub_implementation(
        &mut self,
        sub_name_node: BareNameNode,
        params: DeclaredNameNodes,
        block: parser::StatementNodes,
    ) -> Result<Option<TopLevelToken>, QErrorNode> {
        self.push_sub_context(sub_name_node.as_ref());

        let mut mapped_params: ResolvedDeclaredNameNodes = vec![];
        for declared_name_node in params.into_iter() {
            let Locatable {
                element: declared_name,
                pos,
            } = declared_name_node;
            let bare_name: &BareName = declared_name.as_ref();
            if self.subs.contains_key(bare_name) {
                // not possible to have a param name that clashes with a sub (functions are ok)
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let q: TypeQualifier = self.resolve_declared_name(&declared_name);
            // TODO it is not always CompactBuiltIn, support ExtendedBuiltIn, if it makes a difference
            let resolved_declared_name = ResolvedDeclaredName {
                name: bare_name.clone(),
                type_definition: ResolvedTypeDefinition::CompactBuiltIn(q),
            };
            self.context
                .push_param(declared_name, &self.resolver)
                .with_err_at(pos)?;
            mapped_params.push(resolved_declared_name.at(pos));
        }

        let mapped = TopLevelToken::SubImplementation(SubImplementation {
            name: sub_name_node,
            params: mapped_params,
            body: self.convert(block)?,
        });
        self.pop_context();
        Ok(Some(mapped))
    }

    pub fn resolve_name_in_assignment(&mut self, n: parser::Name) -> Result<LName, QError> {
        if self.context.is_function_context(&n) {
            // trying to assign to the function
            let function_type: TypeQualifier = self.functions.get(n.as_ref()).unwrap().0;
            if n.is_bare_or_of_type(function_type) {
                Ok(LName::Function(n.with_type(function_type)))
            } else {
                // trying to assign to the function with an explicit wrong type
                Err(QError::DuplicateDefinition)
            }
        } else if self.subs.contains_key(n.as_ref()) {
            // trying to assign to a sub
            Err(QError::DuplicateDefinition)
        } else if !self.context.resolve_param_assignment(&n, &self.resolver)?
            && self.functions.contains_key(n.as_ref())
        {
            // parameter might be hiding a function name so it takes precedence
            Err(QError::DuplicateDefinition)
        } else {
            let resolved_declared_name: ResolvedDeclaredName =
                self.context.resolve_assignment(&n, &self.resolver)?;
            Ok(LName::Variable(resolved_declared_name))
        }
    }

    pub fn resolve_name_in_expression(&mut self, n: &parser::Name) -> Result<Expression, QError> {
        self.context
            .resolve_expression(n, &self.resolver)
            .or_try_read(|| self.resolve_name_as_subprogram(n))
            .or_read(|| {
                self.context
                    .resolve_missing_name_in_expression(n, &self.resolver)
            })
    }

    fn resolve_name_as_subprogram(
        &mut self,
        n: &parser::Name,
    ) -> Result<Option<Expression>, QError> {
        if self.subs.contains_key(n.as_ref()) {
            // using the name of a sub as a variable expression
            Err(QError::DuplicateDefinition)
        } else if self.functions.contains_key(n.as_ref()) {
            // if the function expects arguments, argument count mismatch
            let (f_type, f_args, _) = self.functions.get(n.as_ref()).unwrap();
            if !f_args.is_empty() {
                Err(QError::ArgumentCountMismatch)
            } else if !n.is_bare_or_of_type(*f_type) {
                // if the function is a different type and the name is qualified of a different type, duplication definition
                Err(QError::DuplicateDefinition)
            } else {
                // else convert it to function call
                Ok(Some(Expression::FunctionCall(
                    n.with_type_ref(*f_type),
                    vec![],
                )))
            }
        } else {
            Ok(None)
        }
    }

    fn constant(
        &mut self,
        left: NameNode,
        right: crate::parser::ExpressionNode,
    ) -> Result<Statement, QErrorNode> {
        let Locatable { element: name, pos } = left;
        let bare_name: &BareName = name.as_ref();
        if self.functions.contains_key(bare_name)
            || self.subs.contains_key(bare_name)
            || self.context.contains_any(&name)
        {
            // local variable/param or local constant or function or sub already present by that name
            Err(QError::DuplicateDefinition).with_err_at(pos)
        } else {
            // TODO simplify
            let checked_right = self.resolve_const_value(right)?;
            let converted_expression_node = self.convert(checked_right)?;
            match converted_expression_node.try_type_definition()? {
                ResolvedTypeDefinition::CompactBuiltIn(q)
                | ResolvedTypeDefinition::ExtendedBuiltIn(q) => {
                    let q_name = self
                        .context
                        .push_const(name, q.at(converted_expression_node.pos()))
                        .patch_err_pos(pos)?;
                    Ok(Statement::Const(q_name.at(pos), converted_expression_node))
                }
                _ => Err(QError::InvalidConstant).with_err_at(pos),
            }
        }
    }

    fn resolve_const_value(
        &self,
        e: crate::parser::ExpressionNode,
    ) -> Result<crate::parser::ExpressionNode, QErrorNode> {
        let Locatable { element, pos } = e;
        match element {
            crate::parser::Expression::IntegerLiteral(i) => {
                Ok(crate::parser::Expression::IntegerLiteral(i).at(pos))
            }
            crate::parser::Expression::LongLiteral(l) => {
                Ok(crate::parser::Expression::LongLiteral(l).at(pos))
            }
            crate::parser::Expression::SingleLiteral(f) => {
                Ok(crate::parser::Expression::SingleLiteral(f).at(pos))
            }
            crate::parser::Expression::DoubleLiteral(d) => {
                Ok(crate::parser::Expression::DoubleLiteral(d).at(pos))
            }
            crate::parser::Expression::StringLiteral(s) => {
                Ok(crate::parser::Expression::StringLiteral(s).at(pos))
            }
            crate::parser::Expression::BinaryExpression(op, boxed_l, boxed_r) => {
                let l: crate::parser::ExpressionNode = *boxed_l;
                let r: crate::parser::ExpressionNode = *boxed_r;
                let res_l = self.resolve_const_value(l)?;
                let res_r = self.resolve_const_value(r)?;
                // TODO check if casting is applicable and resolve value
                Ok(crate::parser::Expression::BinaryExpression(
                    op,
                    Box::new(res_l),
                    Box::new(res_r),
                )
                .at(pos))
            }
            crate::parser::Expression::UnaryExpression(op, boxed_child) => {
                let child: crate::parser::ExpressionNode = *boxed_child;
                let res = self.resolve_const_value(child)?;
                Ok(crate::parser::Expression::UnaryExpression(op, Box::new(res)).at(pos))
            }
            crate::parser::Expression::Parenthesis(boxed_child) => {
                let child: crate::parser::ExpressionNode = *boxed_child;
                let res = self.resolve_const_value(child)?;
                Ok(crate::parser::Expression::Parenthesis(Box::new(res)).at(pos))
            }
            crate::parser::Expression::VariableName(variable_name) => {
                // if a constant exists in this scope or the parent scope, it is okay
                match self
                    .context
                    .resolve_const_expression(&variable_name)
                    .with_err_at(pos)?
                {
                    Some(_) => Ok(crate::parser::Expression::VariableName(variable_name).at(pos)),
                    None => Err(QError::InvalidConstant).with_err_at(pos),
                }
            }
            crate::parser::Expression::FunctionCall(_, _)
            | crate::parser::Expression::FileHandle(_) => {
                Err(QError::InvalidConstant).with_err_at(pos)
            }
        }
    }

    fn assignment(
        &mut self,
        name: Name,
        expression_node: crate::parser::ExpressionNode,
    ) -> Result<Statement, QErrorNode> {
        let resolved_l_name = self.resolve_name_in_assignment(name).with_err_no_pos()?;
        let converted_expr: ExpressionNode = self.convert(expression_node)?;
        let result_q: ResolvedTypeDefinition = converted_expr.try_type_definition()?;
        match resolved_l_name {
            LName::Variable(ResolvedDeclaredName {
                name,
                type_definition,
            }) => {
                if result_q.can_cast_to(&type_definition) {
                    Ok(Statement::Assignment(
                        ResolvedDeclaredName {
                            name,
                            type_definition,
                        },
                        converted_expr,
                    ))
                } else {
                    Err(QError::TypeMismatch).with_err_at(&converted_expr)
                }
            }
            LName::Function(QualifiedName { qualifier, .. }) => {
                if result_q.can_cast_to(qualifier) {
                    Ok(Statement::SetReturnValue(converted_expr))
                } else {
                    Err(QError::TypeMismatch).with_err_at(&converted_expr)
                }
            }
        }
    }
}

pub fn convert(
    program: parser::ProgramNode,
) -> Result<(ProgramNode, FunctionMap, SubMap), QErrorNode> {
    // first pass
    let mut first_pass = FirstPassOuter::new();
    first_pass.parse(&program)?;
    let (f_c, s_c, user_defined_types) = first_pass.into_inner();
    // second pass
    let mut converter = ConverterImpl::default();
    converter.functions = f_c;
    converter.subs = s_c;
    converter.context.user_defined_types = user_defined_types;
    let result = converter.convert(program)?;
    let (f, s) = converter.consume();
    Ok((result, f, s))
}

impl Converter<parser::ProgramNode, ProgramNode> for ConverterImpl {
    fn convert(&mut self, a: parser::ProgramNode) -> Result<ProgramNode, QErrorNode> {
        let mut result: Vec<TopLevelTokenNode> = vec![];
        for top_level_token_node in a.into_iter() {
            // will contain None where DefInt and declarations used to be
            let Locatable { element, pos } = top_level_token_node;
            let opt: Option<TopLevelToken> = self.convert(element).patch_err_pos(pos)?;
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
    fn convert(&mut self, a: Name) -> Result<QualifiedName, QErrorNode> {
        Ok(a.resolve_into(&self.resolver))
    }
}

// Option because we filter out DefType and UserDefinedType
impl Converter<parser::TopLevelToken, Option<TopLevelToken>> for ConverterImpl {
    fn convert(&mut self, a: parser::TopLevelToken) -> Result<Option<TopLevelToken>, QErrorNode> {
        match a {
            parser::TopLevelToken::DefType(d) => {
                self.resolver.set(&d);
                Ok(None)
            }
            parser::TopLevelToken::FunctionDeclaration(_, _)
            | parser::TopLevelToken::SubDeclaration(_, _) => Ok(None),
            parser::TopLevelToken::FunctionImplementation(n, params, block) => {
                self.convert_function_implementation(n, params, block)
            }
            parser::TopLevelToken::SubImplementation(n, params, block) => {
                self.convert_sub_implementation(n, params, block)
            }
            parser::TopLevelToken::Statement(s) => {
                Ok(Some(TopLevelToken::Statement(self.convert(s)?)))
            }
            parser::TopLevelToken::UserDefinedType(_) => {
                // already handled by first pass
                Ok(None)
            }
        }
    }
}

impl Converter<parser::Statement, Statement> for ConverterImpl {
    fn convert(&mut self, a: parser::Statement) -> Result<Statement, QErrorNode> {
        match a {
            parser::Statement::Comment(c) => Ok(Statement::Comment(c)),
            parser::Statement::Assignment(n, e) => self.assignment(n, e),
            parser::Statement::Const(n, e) => self.constant(n, e),
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
            parser::Statement::Dim(declared_name_node) => {
                let Locatable {
                    element: declared_name,
                    pos,
                } = declared_name_node;
                let bare_name: &BareName = declared_name.as_ref();
                if self.subs.contains_key(bare_name) || self.functions.contains_key(bare_name) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                let mapped_declared_name = self
                    .context
                    .push_dim(declared_name, &self.resolver)
                    .with_err_at(pos)?;
                Ok(Statement::Dim(mapped_declared_name.at(pos)))
            }
        }
    }
}

impl Converter<parser::Expression, Expression> for ConverterImpl {
    fn convert(&mut self, a: parser::Expression) -> Result<Expression, QErrorNode> {
        match a {
            parser::Expression::SingleLiteral(f) => Ok(Expression::SingleLiteral(f)),
            parser::Expression::DoubleLiteral(f) => Ok(Expression::DoubleLiteral(f)),
            parser::Expression::StringLiteral(f) => Ok(Expression::StringLiteral(f)),
            parser::Expression::IntegerLiteral(f) => Ok(Expression::IntegerLiteral(f)),
            parser::Expression::LongLiteral(f) => Ok(Expression::LongLiteral(f)),
            parser::Expression::VariableName(n) => {
                self.resolve_name_in_expression(&n).with_err_no_pos()
            }
            parser::Expression::FunctionCall(n, args) => {
                let converted_args = self.convert(args)?;
                let opt_built_in: Option<BuiltInFunction> = (&n).try_into().with_err_no_pos()?;
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
                match converted_left.try_type_definition()? {
                    ResolvedTypeDefinition::CompactBuiltIn(q_left)
                    | ResolvedTypeDefinition::ExtendedBuiltIn(q_left) => {
                        match converted_right.try_type_definition()? {
                            ResolvedTypeDefinition::CompactBuiltIn(q_right)
                            | ResolvedTypeDefinition::ExtendedBuiltIn(q_right) => {
                                // get the cast type
                                let result_type =
                                    super::casting::cast_binary_op(op, q_left, q_right);
                                if result_type.is_some() {
                                    Ok(Expression::BinaryExpression(
                                        op,
                                        Box::new(converted_left),
                                        Box::new(converted_right),
                                    ))
                                } else {
                                    Err(QError::TypeMismatch).with_err_at(&converted_right)
                                }
                            }
                            _ => Err(QError::TypeMismatch).with_err_at(&converted_right),
                        }
                    }
                    _ => Err(QError::TypeMismatch).with_err_at(&converted_left),
                }
            }
            parser::Expression::UnaryExpression(op, c) => {
                let unboxed_child = *c;
                let converted_child = self.convert(unboxed_child)?;
                match converted_child.try_type_definition()? {
                    ResolvedTypeDefinition::CompactBuiltIn(q)
                    | ResolvedTypeDefinition::ExtendedBuiltIn(q) => {
                        if super::casting::cast_unary_op(op, q).is_none() {
                            Err(QError::TypeMismatch).with_err_at(&converted_child)
                        } else {
                            Ok(Expression::UnaryExpression(op, Box::new(converted_child)))
                        }
                    }
                    _ => Err(QError::TypeMismatch).with_err_at(&converted_child),
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
    fn convert(&mut self, a: parser::ForLoopNode) -> Result<ForLoopNode, QErrorNode> {
        Ok(ForLoopNode {
            variable_name: temp_convert(self.convert(a.variable_name)?),
            lower_bound: self.convert(a.lower_bound)?,
            upper_bound: self.convert(a.upper_bound)?,
            step: self.convert(a.step)?,
            statements: self.convert(a.statements)?,
            next_counter: self.convert(a.next_counter)?.map(temp_convert),
        })
    }
}

fn temp_convert(x: QualifiedNameNode) -> ResolvedDeclaredNameNode {
    let Locatable { element, pos } = x;
    let QualifiedName { name, qualifier } = element;
    ResolvedDeclaredName {
        name,
        type_definition: ResolvedTypeDefinition::CompactBuiltIn(qualifier),
    }
    .at(pos)
}

impl Converter<parser::ConditionalBlockNode, ConditionalBlockNode> for ConverterImpl {
    fn convert(
        &mut self,
        a: parser::ConditionalBlockNode,
    ) -> Result<ConditionalBlockNode, QErrorNode> {
        Ok(ConditionalBlockNode {
            condition: self.convert(a.condition)?,
            statements: self.convert(a.statements)?,
        })
    }
}

impl Converter<parser::IfBlockNode, IfBlockNode> for ConverterImpl {
    fn convert(&mut self, a: parser::IfBlockNode) -> Result<IfBlockNode, QErrorNode> {
        Ok(IfBlockNode {
            if_block: self.convert(a.if_block)?,
            else_if_blocks: self.convert(a.else_if_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}

impl Converter<parser::SelectCaseNode, SelectCaseNode> for ConverterImpl {
    fn convert(&mut self, a: parser::SelectCaseNode) -> Result<SelectCaseNode, QErrorNode> {
        Ok(SelectCaseNode {
            expr: self.convert(a.expr)?,
            case_blocks: self.convert(a.case_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}

impl Converter<parser::CaseBlockNode, CaseBlockNode> for ConverterImpl {
    fn convert(&mut self, a: parser::CaseBlockNode) -> Result<CaseBlockNode, QErrorNode> {
        Ok(CaseBlockNode {
            expr: self.convert(a.expr)?,
            statements: self.convert(a.statements)?,
        })
    }
}

impl Converter<parser::CaseExpression, CaseExpression> for ConverterImpl {
    fn convert(&mut self, a: parser::CaseExpression) -> Result<CaseExpression, QErrorNode> {
        match a {
            parser::CaseExpression::Simple(e) => self.convert(e).map(|x| CaseExpression::Simple(x)),
            parser::CaseExpression::Is(op, e) => self.convert(e).map(|x| CaseExpression::Is(op, x)),
            parser::CaseExpression::Range(from, to) => self
                .convert(from)
                .and_then(|x| self.convert(to).map(|y| CaseExpression::Range(x, y))),
        }
    }
}

#[derive(Debug)]
enum LName {
    Variable(ResolvedDeclaredName),
    Function(QualifiedName),
}
