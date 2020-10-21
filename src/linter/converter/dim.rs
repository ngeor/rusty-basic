use crate::common::{AtLocation, Locatable, QError, QErrorNode, ToLocatableError};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::type_resolver::TypeResolver;
use crate::linter::{DimName, DimNameNode, DimType};
use crate::parser;
use crate::variant::Variant;

impl<'a> Converter<parser::DimNameNode, DimNameNode> for ConverterImpl<'a> {
    fn convert(&mut self, dim_name_node: parser::DimNameNode) -> Result<DimNameNode, QErrorNode> {
        let Locatable { element, pos } = dim_name_node;
        let (bare_name, dim_type) = element.into_inner();
        if self.subs.contains_key(&bare_name)
            || self.functions.contains_key(&bare_name)
            || self.context.contains_const(&bare_name) | self.context.contains_extended(&bare_name)
        {
            return Err(QError::DuplicateDefinition).with_err_at(pos);
        }
        let dim_type: DimType = match dim_type {
            parser::DimType::Bare => {
                let q = self.resolver.resolve(&bare_name);
                if self.context.contains_compact(&bare_name, q) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                self.context.push_dim_compact(bare_name.clone(), q);
                DimType::BuiltIn(q)
            }
            parser::DimType::Compact(q) => {
                if self.context.contains_compact(&bare_name, q) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                self.context.push_dim_compact(bare_name.clone(), q);
                DimType::BuiltIn(q)
            }
            parser::DimType::FixedLengthString(len_expr) => {
                if self.context.contains_any(&bare_name) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                let len: u16 = match self.context.resolve_const_value_node(&len_expr)? {
                    Variant::VInteger(i) => i as u16,
                    _ => {
                        return Err(QError::ArgumentTypeMismatch).with_err_at(&len_expr);
                    }
                };
                self.context.push_dim_string(bare_name.clone(), len);
                DimType::FixedLengthString(len)
            }
            parser::DimType::Extended(q) => {
                if self.context.contains_any(&bare_name) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                self.context.push_dim_extended(bare_name.clone(), q);
                DimType::BuiltIn(q)
            }
            parser::DimType::UserDefined(Locatable {
                element: type_name, ..
            }) => {
                if self.context.contains_any(&bare_name) {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
                if !self.user_defined_types.contains_key(&type_name) {
                    return Err(QError::TypeNotDefined).with_err_at(pos);
                }
                self.context
                    .push_dim_user_defined(bare_name.clone(), type_name.clone());
                DimType::UserDefined(type_name)
            }
            parser::DimType::Array(_dimensions, _box_type) => todo!(),
        };

        Ok(DimName::new(bare_name, dim_type).at(pos))
    }
}
