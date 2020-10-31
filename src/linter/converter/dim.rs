use crate::common::{
    AtLocation, HasLocation, Locatable, PatchErrPos, QError, QErrorNode, ToErrorEnvelopeNoPos,
    ToLocatableError,
};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::type_resolver::TypeResolver;
use crate::linter::{ArrayDimension, DimName, DimNameNode, DimType, Expression};
use crate::parser;
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};
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
        let dim_type: DimType = self
            .convert_dim_type(&bare_name, dim_type)
            .patch_err_pos(pos)?;
        Ok(DimName::new(bare_name, dim_type).at(pos))
    }
}

impl<'a> ConverterImpl<'a> {
    fn convert_dim_type(
        &mut self,
        bare_name: &BareName,
        dim_type: parser::DimType,
    ) -> Result<DimType, QErrorNode> {
        match dim_type {
            parser::DimType::Bare => self.convert_dim_type_bare(bare_name).with_err_no_pos(),
            parser::DimType::Compact(q) => self
                .convert_dim_type_compact(bare_name, q)
                .with_err_no_pos(),
            parser::DimType::FixedLengthString(len_expr) => {
                self.convert_dim_type_fixed_length_string(bare_name, len_expr)
            }
            parser::DimType::Extended(q) => self
                .convert_dim_type_extended(bare_name, q)
                .with_err_no_pos(),
            parser::DimType::UserDefined(user_defined_type_node) => {
                self.convert_dim_type_user_defined(bare_name, user_defined_type_node)
            }
            parser::DimType::Array(dimensions, box_type) => {
                self.convert_dim_type_array(bare_name, dimensions, *box_type)
            }
        }
    }

    fn convert_dim_type_bare(&mut self, bare_name: &BareName) -> Result<DimType, QError> {
        let q = self.resolver.resolve(&bare_name);
        if self.context.contains_compact(&bare_name, q) {
            return Err(QError::DuplicateDefinition);
        }
        self.context.push_dim_compact(bare_name.clone(), q);
        Ok(DimType::BuiltIn(q))
    }

    fn convert_dim_type_compact(
        &mut self,
        bare_name: &BareName,
        q: TypeQualifier,
    ) -> Result<DimType, QError> {
        if self.context.contains_compact(&bare_name, q) {
            return Err(QError::DuplicateDefinition);
        }
        self.context.push_dim_compact(bare_name.clone(), q);
        Ok(DimType::BuiltIn(q))
    }

    fn convert_dim_type_fixed_length_string(
        &mut self,
        bare_name: &BareName,
        len_expr: parser::ExpressionNode,
    ) -> Result<DimType, QErrorNode> {
        if self.context.contains_any(&bare_name) {
            return Err(QError::DuplicateDefinition).with_err_no_pos();
        }
        let len: u16 = match self.context.resolve_const_value_node(&len_expr)? {
            Variant::VInteger(i) => i as u16,
            _ => {
                return Err(QError::ArgumentTypeMismatch).with_err_at(&len_expr);
            }
        };
        self.context.push_dim_string(bare_name.clone(), len);
        Ok(DimType::FixedLengthString(len))
    }

    fn convert_dim_type_extended(
        &mut self,
        bare_name: &BareName,
        q: TypeQualifier,
    ) -> Result<DimType, QError> {
        if self.context.contains_any(&bare_name) {
            return Err(QError::DuplicateDefinition);
        }
        self.context.push_dim_extended(bare_name.clone(), q);
        Ok(DimType::BuiltIn(q))
    }

    fn convert_dim_type_user_defined(
        &mut self,
        bare_name: &BareName,
        user_defined_type: parser::BareNameNode,
    ) -> Result<DimType, QErrorNode> {
        if self.context.contains_any(&bare_name) {
            return Err(QError::DuplicateDefinition).with_err_no_pos();
        }
        let Locatable {
            element: type_name,
            pos,
        } = user_defined_type;
        if !self.user_defined_types.contains_key(&type_name) {
            return Err(QError::TypeNotDefined).with_err_at(pos);
        }
        self.context
            .push_dim_user_defined(bare_name.clone(), type_name.clone());
        Ok(DimType::UserDefined(type_name))
    }

    fn convert_dim_type_array(
        &mut self,
        bare_name: &BareName,
        array_dimensions: parser::ArrayDimensions,
        element_type: parser::DimType,
    ) -> Result<DimType, QErrorNode> {
        // re-construct declared name
        let declared_name: Name = match &element_type {
            parser::DimType::Compact(q) => {
                Name::Qualified(QualifiedName::new(bare_name.clone(), *q))
            }
            _ => Name::Bare(bare_name.clone()),
        };

        let converted_element_type = self.convert_dim_type(bare_name, element_type)?;
        let converted_array_dimensions = self.convert(array_dimensions)?;
        let dim_type = DimType::Array(
            converted_array_dimensions.clone(),
            Box::new(converted_element_type),
        );
        self.context
            .register_array_dimensions(declared_name, converted_array_dimensions);
        Ok(dim_type)
    }
}

impl<'a> Converter<parser::ArrayDimension, ArrayDimension> for ConverterImpl<'a> {
    fn convert(
        &mut self,
        array_dimension: parser::ArrayDimension,
    ) -> Result<ArrayDimension, QErrorNode> {
        let parser::ArrayDimension { lbound, ubound } = array_dimension;
        match lbound {
            Some(lbound) => {
                let converted_lbound = self.convert(lbound)?;
                let converted_ubound = self.convert(ubound)?;
                Ok(ArrayDimension {
                    lbound: converted_lbound,
                    ubound: converted_ubound,
                })
            }
            None => {
                let converted_lbound = Expression::IntegerLiteral(0).at(ubound.pos());
                let converted_ubound = self.convert(ubound)?;
                Ok(ArrayDimension {
                    lbound: converted_lbound,
                    ubound: converted_ubound,
                })
            }
        }
    }
}
