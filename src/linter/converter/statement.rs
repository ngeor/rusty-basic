use super::converter::{Converter, ConverterImpl};
use crate::built_ins::BuiltInSub;
use crate::common::*;
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::types::{DimNameNode, ExpressionNode, Statement};
use crate::linter::{DimName, UserDefinedName};
use crate::parser;
use crate::parser::{BareName, Name, NameNode, QualifiedName, TypeQualifier};
use crate::variant::Variant;
use std::convert::TryInto;

impl<'a> Converter<parser::Statement, Statement> for ConverterImpl<'a> {
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
            parser::Statement::Dim(dim_name_node) => {
                self.convert(dim_name_node).map(|x| Statement::Dim(x))
            }
        }
    }
}

impl<'a> Converter<parser::DimNameNode, DimNameNode> for ConverterImpl<'a> {
    fn convert(&mut self, dim_name_node: parser::DimNameNode) -> Result<DimNameNode, QErrorNode> {
        let Locatable { element, pos } = dim_name_node;
        let bare_name: &BareName = element.as_ref();
        if self.subs.contains_key(bare_name)
            || self.functions.contains_key(bare_name)
            || self.context.contains_const(bare_name) | self.context.contains_extended(bare_name)
        {
            return Err(QError::DuplicateDefinition).with_err_at(pos);
        }
        let n: DimName = match element {
            parser::DimName::Bare(b) => {
                let q = self.resolver.resolve(&b);
                if self.context.contains_compact(&b, q) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                self.context.push_dim_compact(b.clone(), q);
                DimName::BuiltIn(b, q)
            }
            parser::DimName::Compact(b, q) => {
                if self.context.contains_compact(&b, q) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                self.context.push_dim_compact(b.clone(), q);
                DimName::BuiltIn(b, q)
            }
            parser::DimName::String(b, len_expr) => {
                if self.context.contains_any(&b) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                let len: u16 = match self.context.resolve_const_value_node(&len_expr)? {
                    Variant::VInteger(i) => i as u16,
                    _ => {
                        return Err(QError::ArgumentTypeMismatch).with_err_at(len_expr);
                    }
                };
                self.context.push_dim_string(b.clone(), len);
                DimName::String(b, len)
            }
            parser::DimName::ExtendedBuiltIn(b, q) => {
                if self.context.contains_any(&b) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                self.context.push_dim_extended(b.clone(), q);
                DimName::BuiltIn(b, q)
            }
            parser::DimName::UserDefined(b, u) => {
                if self.context.contains_any(&b) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                if !self.user_defined_types.contains_key(&u) {
                    return Err(QError::TypeNotDefined).with_err_at(pos);
                }
                if b.contains('.') {
                    return Err(QError::IdentifierCannotIncludePeriod).with_err_at(pos);
                }
                self.context.push_dim_user_defined(b.clone(), u.clone());
                DimName::UserDefined(UserDefinedName {
                    name: b,
                    type_name: u,
                })
            }
        };

        Ok(n.at(pos))
    }
}

impl<'a> ConverterImpl<'a> {
    fn assignment(
        &mut self,
        name: Name,
        expression_node: crate::parser::ExpressionNode,
    ) -> Result<Statement, QErrorNode> {
        let resolved_l_name = self.resolve_name_in_assignment(name).with_err_no_pos()?;
        let converted_expr: ExpressionNode = self.convert(expression_node)?;
        if converted_expr.can_cast_to(&resolved_l_name) {
            Ok(Statement::Assignment(resolved_l_name, converted_expr))
        } else {
            Err(QError::TypeMismatch).with_err_at(&converted_expr)
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
            || self.context.contains_any(bare_name)
        {
            // local variable/param or local constant or function or sub already present by that name
            Err(QError::DuplicateDefinition).with_err_at(pos)
        } else {
            let v = self.context.resolve_const_value_node(&right)?;
            match name {
                Name::Bare(b) => {
                    // type comes from the right side, not the resolver
                    let q: TypeQualifier = (&v).try_into().with_err_at(right)?;
                    self.context.push_const(b.clone(), q, v.clone());
                    Ok(Statement::Const(QualifiedName::new(b, q).at(pos), v))
                }
                Name::Qualified {
                    bare_name: name,
                    qualifier,
                } => {
                    // type comes from the left side + casting
                    let casted_v = v.cast(qualifier).with_err_at(right)?;
                    self.context
                        .push_const(name.clone(), qualifier, casted_v.clone());
                    Ok(Statement::Const(
                        QualifiedName::new(name, qualifier).at(pos),
                        casted_v,
                    ))
                }
            }
        }
    }
}
