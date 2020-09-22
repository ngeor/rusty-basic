use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::linter::converter::context::LinterContext;
use crate::linter::type_resolver::*;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::types::*;
use crate::parser;
use crate::parser::{
    BareName, BareNameNode, HasQualifier, Name, NameNode, QualifiedName, TypeQualifier,
};
use std::collections::HashSet;
use std::convert::TryInto;

//
// Converter trait
//

pub trait Converter<A, B> {
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

#[derive(Debug)]
pub struct ConverterImpl<'a> {
    pub resolver: TypeResolverImpl,
    pub context: LinterContext<'a>,
    pub functions: &'a FunctionMap,
    pub subs: &'a SubMap,
    pub user_defined_types: &'a UserDefinedTypes,
}

impl<'a> ConverterImpl<'a> {
    pub fn new(
        user_defined_types: &'a UserDefinedTypes,
        functions: &'a FunctionMap,
        subs: &'a SubMap,
    ) -> Self {
        Self {
            user_defined_types,
            resolver: TypeResolverImpl::new(),
            context: LinterContext::new(user_defined_types),
            functions,
            subs,
        }
    }

    fn take_context(&mut self) -> LinterContext<'a> {
        let tmp = LinterContext::new(&self.user_defined_types);
        std::mem::replace(&mut self.context, tmp)
    }

    pub fn push_function_context(&mut self, name: &CaseInsensitiveString) {
        let old = self.take_context();
        self.context = old.push_function_context(name);
    }

    pub fn push_sub_context(&mut self, name: &CaseInsensitiveString) {
        let old = self.take_context();
        self.context = old.push_sub_context(name);
    }

    pub fn pop_context(&mut self) {
        let old = self.take_context();
        self.context = old.pop_context();
    }

    pub fn consume(self) -> HashSet<BareName> {
        self.context.names_without_dot.unwrap()
    }

    fn convert_function_implementation(
        &mut self,
        function_name_node: NameNode,
        params: parser::ParamNodes,
        block: parser::StatementNodes,
    ) -> Result<Option<TopLevelToken>, QErrorNode> {
        let mapped_name = function_name_node.map(|x| x.resolve_into(&self.resolver));
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

    // TODO trait FromWithContext fn from(other, &ctx) -> Self
    // TODO more types e.g. ParamName
    // TODO the bool represents extended type, improve this
    // TODO use linter context there are a lot of similarities
    fn resolve_declared_parameter_name(
        &mut self,
        param: &parser::Param,
    ) -> Result<(ResolvedParamName, bool), QError> {
        match param {
            parser::Param::Bare(name) => {
                let q: TypeQualifier = name.resolve_into(&self.resolver);
                Ok((ResolvedParamName::BuiltIn(name.clone(), q), false))
            }
            parser::Param::Compact(name, q) => {
                Ok((ResolvedParamName::BuiltIn(name.clone(), *q), false))
            }
            parser::Param::ExtendedBuiltIn(name, q) => {
                Ok((ResolvedParamName::BuiltIn(name.clone(), *q), true))
            }
            parser::Param::UserDefined(name, u) => {
                if self.user_defined_types.contains_key(u) {
                    Ok((
                        ResolvedParamName::UserDefined(name.clone(), u.clone()),
                        true,
                    ))
                } else {
                    Err(QError::TypeNotDefined)
                }
            }
        }
    }

    fn convert_function_params(
        &mut self,
        function_name: &QualifiedName,
        params: parser::ParamNodes,
    ) -> Result<Vec<Locatable<ResolvedParamName>>, QErrorNode> {
        let mut result: Vec<Locatable<ResolvedParamName>> = vec![];
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
            let (resolved_declared_name, is_extended) = self
                .resolve_declared_parameter_name(&declared_name)
                .with_err_at(pos)?;
            let bare_function_name: &BareName = function_name.as_ref();
            if bare_function_name == bare_name {
                // not possible to have a param name clashing with the function name if the type is different or if it's an extended declaration (AS SINGLE)
                let clashes = match &resolved_declared_name {
                    ResolvedParamName::BuiltIn(_, qualifier) => {
                        *qualifier != function_name.qualifier() || is_extended
                    }
                    _ => true,
                };
                if clashes {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
            }
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
        params: parser::ParamNodes,
        block: parser::StatementNodes,
    ) -> Result<Option<TopLevelToken>, QErrorNode> {
        self.push_sub_context(sub_name_node.as_ref());

        let mut mapped_params: Vec<Locatable<ResolvedParamName>> = vec![];
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
            let (resolved_declared_name, _) = self
                .resolve_declared_parameter_name(&declared_name)
                .with_err_at(pos)?;
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

    pub fn resolve_name_in_assignment(&mut self, n: Name) -> Result<ResolvedDeclaredName, QError> {
        let bare_name: &BareName = n.as_ref();
        if self.context.is_function_context(&n) {
            // trying to assign to the function
            let Locatable {
                element: (function_type, _),
                ..
            } = self.functions.get(bare_name).unwrap();
            if n.is_bare_or_of_type(*function_type) {
                Ok(ResolvedDeclaredName::BuiltIn(QualifiedName::new(
                    bare_name.clone(),
                    *function_type,
                )))
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
            Ok(resolved_declared_name)
        }
    }

    pub fn resolve_name_in_expression(&mut self, n: &Name) -> Result<Expression, QError> {
        // TODO function context should upfront have an implicit variable equal to the function name which can be referenced
        // as bare or as the correct type and cannot be shadowed by a variable of different type
        match self.context.resolve_expression(n, &self.resolver)? {
            Some(x) => Ok(x),
            None => match self.resolve_name_as_subprogram(n)? {
                Some(x) => Ok(x),
                None => self
                    .context
                    .resolve_missing_name_in_expression(n, &self.resolver),
            },
        }
    }

    fn resolve_name_as_subprogram(&mut self, n: &Name) -> Result<Option<Expression>, QError> {
        if self.subs.contains_key(n.as_ref()) {
            // using the name of a sub as a variable expression
            Err(QError::DuplicateDefinition)
        } else if self.functions.contains_key(n.as_ref()) {
            // if the function expects arguments, argument count mismatch
            let Locatable {
                element: (f_type, f_args),
                ..
            } = self.functions.get(n.as_ref()).unwrap();
            if !f_args.is_empty() {
                Err(QError::ArgumentCountMismatch)
            } else if self.context.is_function_context(n) {
                // We are inside a function that takes no args, and we're using again
                // the name of that function as an expression.
                // This can only work as a variable, otherwise we'll get infinite recursive call.
                //
                // Example:
                // Function Test
                //     INPUT Test
                // End Function
                //
                // Return None and let the next handler add it as a new variable
                Ok(None)
            } else {
                match n {
                    Name::Bare(b) => Ok(Some(Expression::FunctionCall(
                        QualifiedName::new(b.clone(), *f_type),
                        vec![],
                    ))),
                    Name::Qualified { name, qualifier } => {
                        // if the function is a different type and the name is qualified of a different type, duplication definition
                        if f_type != qualifier {
                            Err(QError::DuplicateDefinition)
                        } else {
                            Ok(Some(Expression::FunctionCall(
                                QualifiedName::new(name.clone(), *f_type),
                                vec![],
                            )))
                        }
                    }
                }
            }
        } else {
            Ok(None)
        }
    }

    // TODO fix me
    fn temp_convert(&mut self, x: NameNode) -> Result<ResolvedDeclaredName, QErrorNode> {
        let Locatable { element, pos } = x;
        self.resolve_name_in_assignment(element).with_err_at(pos)
    }
}

impl<'a> Converter<parser::ProgramNode, ProgramNode> for ConverterImpl<'a> {
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

// Option because we filter out DefType and UserDefinedType
impl<'a> Converter<parser::TopLevelToken, Option<TopLevelToken>> for ConverterImpl<'a> {
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

impl<'a> Converter<parser::Expression, Expression> for ConverterImpl<'a> {
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
                    None => Ok(Expression::FunctionCall(
                        n.resolve_into(&self.resolver),
                        converted_args,
                    )),
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
                let t_left = converted_left.type_definition();
                let t_right = converted_right.type_definition();
                // get the cast type
                match t_left.cast_binary_op(t_right, op) {
                    Some(type_definition) => Ok(Expression::BinaryExpression(
                        op,
                        Box::new(converted_left),
                        Box::new(converted_right),
                        type_definition,
                    )),
                    None => Err(QError::TypeMismatch).with_err_at(&converted_right),
                }
            }
            parser::Expression::UnaryExpression(op, c) => {
                let unboxed_child = *c;
                let converted_child = self.convert(unboxed_child)?;
                match converted_child.type_definition() {
                    TypeDefinition::BuiltIn(TypeQualifier::DollarString) => {
                        Err(QError::TypeMismatch).with_err_at(&converted_child)
                    }
                    TypeDefinition::BuiltIn(_) => {
                        Ok(Expression::UnaryExpression(op, Box::new(converted_child)))
                    }
                    // user defined cannot be in unary expressions
                    _ => Err(QError::TypeMismatch).with_err_no_pos(),
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

impl<'a> Converter<parser::ForLoopNode, ForLoopNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::ForLoopNode) -> Result<ForLoopNode, QErrorNode> {
        Ok(ForLoopNode {
            variable_name: self.temp_convert(a.variable_name)?,
            lower_bound: self.convert(a.lower_bound)?,
            upper_bound: self.convert(a.upper_bound)?,
            step: self.convert(a.step)?,
            statements: self.convert(a.statements)?,
            next_counter: match a.next_counter {
                Some(x) => {
                    let pos = x.pos();
                    Some(self.temp_convert(x)?.at(pos))
                }
                None => None,
            },
        })
    }
}

impl<'a> Converter<parser::ConditionalBlockNode, ConditionalBlockNode> for ConverterImpl<'a> {
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

impl<'a> Converter<parser::IfBlockNode, IfBlockNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::IfBlockNode) -> Result<IfBlockNode, QErrorNode> {
        Ok(IfBlockNode {
            if_block: self.convert(a.if_block)?,
            else_if_blocks: self.convert(a.else_if_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}

impl<'a> Converter<parser::SelectCaseNode, SelectCaseNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::SelectCaseNode) -> Result<SelectCaseNode, QErrorNode> {
        Ok(SelectCaseNode {
            expr: self.convert(a.expr)?,
            case_blocks: self.convert(a.case_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}

impl<'a> Converter<parser::CaseBlockNode, CaseBlockNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::CaseBlockNode) -> Result<CaseBlockNode, QErrorNode> {
        Ok(CaseBlockNode {
            expr: self.convert(a.expr)?,
            statements: self.convert(a.statements)?,
        })
    }
}

impl<'a> Converter<parser::CaseExpression, CaseExpression> for ConverterImpl<'a> {
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
