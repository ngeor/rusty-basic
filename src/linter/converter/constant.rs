use crate::common::{AtLocation, Locatable, QError, QErrorNode, ToLocatableError};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::converter::ConverterImpl;
use crate::linter::{Expression, Statement};
use crate::parser::{BareName, Name, NameNode, QualifiedName, TypeQualifier};
use crate::variant::Variant;
use std::convert::TryInto;

impl<'a> ConverterImpl<'a> {
    pub fn constant(
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
                    Ok(Statement::Const(
                        Name::new(b, Some(q)).at(pos),
                        variant_to_expression(v.clone()).at(pos),
                        v,
                    ))
                }
                Name::Qualified(QualifiedName {
                    bare_name: name,
                    qualifier,
                }) => {
                    // type comes from the left side + casting
                    let casted_v = v.cast(qualifier).with_err_at(right)?;
                    self.context
                        .push_const(name.clone(), qualifier, casted_v.clone());
                    Ok(Statement::Const(
                        Name::new(name, Some(qualifier)).at(pos),
                        variant_to_expression(casted_v.clone()).at(pos),
                        casted_v,
                    ))
                }
            }
        }
    }
}

fn variant_to_expression(v: Variant) -> Expression {
    match v {
        Variant::VSingle(f) => Expression::SingleLiteral(f),
        Variant::VDouble(d) => Expression::DoubleLiteral(d),
        Variant::VString(s) => Expression::StringLiteral(s),
        Variant::VInteger(i) => Expression::IntegerLiteral(i),
        Variant::VLong(l) => Expression::LongLiteral(l),
        _ => unimplemented!(),
    }
}
