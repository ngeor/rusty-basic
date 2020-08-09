use super::subprogram_context::{collect_subprograms, FunctionMap, SubMap};
use super::types::*;
use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::linter::linter_context::LinterContext;
use crate::linter::type_resolver::*;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser;
use crate::parser::{
    BareName, BareNameNode, DeclaredName, DeclaredNameNodes, HasQualifier, Name, NameNode,
    QualifiedName, TypeDefinition, TypeQualifier, WithTypeQualifier,
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

pub fn convert(
    program: parser::ProgramNode,
) -> Result<(ProgramNode, FunctionMap, SubMap), QErrorNode> {
    let mut linter = ConverterImpl::default();
    let (f_c, s_c) = collect_subprograms(&program)?;
    linter.functions = f_c;
    linter.subs = s_c;
    let result = linter.convert(program)?;
    let (f, s) = linter.consume();
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

impl ConverterImpl {
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
    ) -> Result<Vec<QNameNode>, QErrorNode> {
        let mut result: Vec<QNameNode> = vec![];
        for p in params.into_iter() {
            let Locatable { element, pos } = p;
            if self.subs.contains_key(element.as_ref()) {
                // not possible to have a param name that clashes with a sub (functions are ok)
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let q: TypeQualifier = self.resolve_declared_name(&element);
            if function_name.as_ref() == element.as_ref()
                && (function_name.qualifier() != q || element.is_extended())
            {
                // not possible to have a param name clashing with the function name if the type is different or if it's an extended declaration (AS SINGLE)
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let q_name = QualifiedName::new(element.as_ref().clone(), q);
            self.context
                .push_param(element, &self.resolver)
                .patch_err_pos(pos)?;
            result.push(q_name.at(pos));
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

        let mut mapped_params: Vec<QNameNode> = vec![];
        for declared_name_node in params.into_iter() {
            let Locatable { element, pos } = declared_name_node;
            if self.subs.contains_key(element.as_ref()) {
                // not possible to have a param name that clashes with a sub (functions are ok)
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let q: TypeQualifier = self.resolve_declared_name(&element);
            let q_name = QualifiedName::new(element.as_ref().clone(), q);
            self.context
                .push_param(element, &self.resolver)
                .patch_err_pos(pos)?;
            mapped_params.push(q_name.at(pos));
        }

        let mapped = TopLevelToken::SubImplementation(SubImplementation {
            name: sub_name_node,
            params: mapped_params,
            body: self.convert(block)?,
        });
        self.pop_context();
        Ok(Some(mapped))
    }
}

// Option because we filter out DefType
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
        }
    }
}

impl Converter<parser::Statement, Statement> for ConverterImpl {
    fn convert(&mut self, a: parser::Statement) -> Result<Statement, QErrorNode> {
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
                    Err(QError::TypeMismatch).with_err_at(&converted_expr)
                }
            }
            parser::Statement::Const(n, e) => {
                let Locatable { element: name, pos } = n;
                if self.functions.contains_key(name.as_ref())
                    || self.subs.contains_key(name.as_ref())
                {
                    // local variable or local constant or function or sub already present by that name
                    Err(QError::DuplicateDefinition).with_err_at(pos)
                } else {
                    let converted_expression_node = self.convert(e)?;
                    let e_type = converted_expression_node.try_qualifier()?;
                    let q_name = self
                        .context
                        .push_const(name, e_type.at(converted_expression_node.pos()))
                        .patch_err_pos(pos)?;
                    Ok(Statement::Const(q_name.at(pos), converted_expression_node))
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
            parser::Statement::Dim(declared_name_node) => {
                let Locatable { element: d, pos } = declared_name_node;
                if self.subs.contains_key(d.as_ref()) || self.functions.contains_key(d.as_ref()) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                let mapped_declared_name = self
                    .context
                    .push_dim(d, &self.resolver)
                    .patch_err_pos(pos)?;
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
            parser::Expression::VariableName(n) => self.resolve_name_in_expression(&n),
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
                    Err(QError::TypeMismatch).with_err_at(&converted_right)
                }
            }
            parser::Expression::UnaryExpression(op, c) => {
                let unboxed_child = *c;
                let converted_child = self.convert(unboxed_child)?;
                let converted_q = converted_child.try_qualifier()?;
                if super::operand_type::cast_unary_op(op, converted_q).is_none() {
                    // no unary operation works for strings
                    Err(QError::TypeMismatch).with_err_at(&converted_child)
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
    fn convert(&mut self, a: parser::ForLoopNode) -> Result<ForLoopNode, QErrorNode> {
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
    pub fn resolve_name_in_assignment(&mut self, n: parser::Name) -> Result<LName, QErrorNode> {
        if self.context.is_function_context(&n) {
            // trying to assign to the function
            let function_type: TypeQualifier = self.functions.get(n.as_ref()).unwrap().0;
            if n.is_bare_or_of_type(function_type) {
                Ok(LName::Function(n.with_type(function_type)))
            } else {
                // trying to assign to the function with an explicit wrong type
                Err(QError::DuplicateDefinition).with_err_no_pos()
            }
        } else if self.subs.contains_key(n.as_ref()) {
            // trying to assign to a sub
            Err(QError::DuplicateDefinition).with_err_no_pos()
        } else if !self.context.resolve_param_assignment(&n, &self.resolver)?
            && self.functions.contains_key(n.as_ref())
        {
            // parameter might be hiding a function name so it takes precedence
            Err(QError::DuplicateDefinition).with_err_no_pos()
        } else {
            let declared_name = self.context.resolve_assignment(&n, &self.resolver)?;
            let q = self.resolve_declared_name(&declared_name);
            let DeclaredName { name, .. } = declared_name;
            Ok(LName::Variable(name.with_type(q)))
        }
    }

    pub fn resolve_name_in_expression(
        &mut self,
        n: &parser::Name,
    ) -> Result<Expression, QErrorNode> {
        self.context
            .resolve_expression(n, &self.resolver)
            .or_try_read(|| self.resolve_name_as_subprogram(n).with_err_no_pos())
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
}
