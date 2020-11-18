use crate::common::{
    AtLocation, Locatable, PatchErrPos, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError,
};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::linter::type_resolver::TypeResolver;
use crate::parser::{
    ArrayDimension, ArrayDimensions, BareName, BareNameNode, BuiltInStyle, DimName, DimNameNode,
    DimType, Expression, ExpressionNode, HasExpressionType, Name, QualifiedNameNode, TypeQualifier,
};
use crate::variant::Variant;

impl<'a> ConverterWithImplicitVariables<DimNameNode, DimNameNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        dim_name_node: DimNameNode,
    ) -> Result<(DimNameNode, Vec<QualifiedNameNode>), QErrorNode> {
        let Locatable { element, pos } = dim_name_node;
        let (bare_name, dim_type) = element.into_inner();
        if self.subs.contains_key(&bare_name)
            || self.functions.contains_key(&bare_name)
            || self.context.contains_const(&bare_name) | self.context.contains_extended(&bare_name)
        {
            return Err(QError::DuplicateDefinition).with_err_at(pos);
        }
        let (dim_type, implicit_variables) = self
            .convert_dim_type(&bare_name, dim_type)
            .patch_err_pos(pos)?;
        Ok((
            DimName::new(bare_name, dim_type).at(pos),
            implicit_variables,
        ))
    }
}

impl<'a> ConverterImpl<'a> {
    fn convert_dim_type(
        &mut self,
        bare_name: &BareName,
        dim_type: DimType,
    ) -> Result<(DimType, Vec<QualifiedNameNode>), QErrorNode> {
        match dim_type {
            DimType::Bare => self
                .convert_dim_type_bare(bare_name)
                .map(|dim_type| (dim_type, vec![]))
                .with_err_no_pos(),
            DimType::BuiltIn(q, BuiltInStyle::Compact) => self
                .convert_dim_type_compact(bare_name, q)
                .map(|dim_type| (dim_type, vec![]))
                .with_err_no_pos(),
            DimType::FixedLengthString(len_expr, _) => self
                .convert_dim_type_fixed_length_string(bare_name, len_expr)
                .map(|dim_type| (dim_type, vec![])),
            DimType::BuiltIn(q, BuiltInStyle::Extended) => self
                .convert_dim_type_extended(bare_name, q)
                .map(|dim_type| (dim_type, vec![]))
                .with_err_no_pos(),
            DimType::UserDefined(user_defined_type_node) => self
                .convert_dim_type_user_defined(bare_name, user_defined_type_node)
                .map(|dim_type| (dim_type, vec![])),
            DimType::Array(dimensions, box_type) => {
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
        Ok(DimType::BuiltIn(q, BuiltInStyle::Compact))
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
        Ok(DimType::BuiltIn(q, BuiltInStyle::Compact))
    }

    fn convert_dim_type_fixed_length_string(
        &mut self,
        bare_name: &BareName,
        len_expr: ExpressionNode,
    ) -> Result<DimType, QErrorNode> {
        if self.context.contains_any(&bare_name) {
            return Err(QError::DuplicateDefinition).with_err_no_pos();
        }
        let len: u16 = self.eval_fixed_string_length(&len_expr)?;
        self.context.push_dim_string(bare_name.clone(), len);
        Ok(DimType::FixedLengthString(
            Expression::IntegerLiteral(len as i32).at(len_expr.pos),
            len,
        ))
    }

    fn eval_fixed_string_length(&self, len_expr: &ExpressionNode) -> Result<u16, QErrorNode> {
        match self.context.resolve_const_value_node(len_expr)? {
            Variant::VInteger(i) => Ok(i as u16),
            _ => Err(QError::ArgumentTypeMismatch).with_err_at(len_expr),
        }
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
        Ok(DimType::BuiltIn(q, BuiltInStyle::Extended))
    }

    fn convert_dim_type_user_defined(
        &mut self,
        bare_name: &BareName,
        user_defined_type: BareNameNode,
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
        Ok(DimType::UserDefined(type_name.at(pos)))
    }

    fn convert_dim_type_array(
        &mut self,
        bare_name: &BareName,
        array_dimensions: ArrayDimensions,
        element_type: DimType,
    ) -> Result<(DimType, Vec<QualifiedNameNode>), QErrorNode> {
        // re-construct declared name
        let resolved_name: Name = match &element_type {
            DimType::Bare => Name::new(bare_name.clone(), Some(self.resolver.resolve(bare_name))),
            DimType::BuiltIn(q, _) => Name::new(bare_name.clone(), Some(*q)),
            _ => Name::Bare(bare_name.clone()),
        };
        let converted_element_type: DimType = match element_type {
            DimType::Bare => {
                DimType::BuiltIn(resolved_name.qualifier().unwrap(), BuiltInStyle::Compact)
            }
            DimType::FixedLengthString(len_expr, _) => {
                let len = self.eval_fixed_string_length(&len_expr)?;
                DimType::FixedLengthString(
                    Expression::IntegerLiteral(len as i32).at(&len_expr),
                    len,
                )
            }
            DimType::Array(_, _) => panic!("Nested array type not supported"),
            _ => element_type,
        };
        let (converted_array_dimensions, implicit_variables) =
            self.convert_and_collect_implicit_variables(array_dimensions)?;
        let dim_type = DimType::Array(
            converted_array_dimensions.clone(),
            Box::new(converted_element_type.clone()),
        );
        self.context.register_array_dimensions(
            resolved_name,
            converted_element_type.expression_type(),
            true,
        );
        Ok((dim_type, implicit_variables))
    }
}

impl<'a> ConverterWithImplicitVariables<ArrayDimension, ArrayDimension> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        array_dimension: ArrayDimension,
    ) -> Result<(ArrayDimension, Vec<QualifiedNameNode>), QErrorNode> {
        let ArrayDimension { lbound, ubound } = array_dimension;
        match lbound {
            Some(lbound) => {
                let (converted_lbound, mut lbound_implicit_variables) =
                    self.convert_and_collect_implicit_variables(lbound)?;
                let (converted_ubound, mut ubound_implicit_variables) =
                    self.convert_and_collect_implicit_variables(ubound)?;
                lbound_implicit_variables.append(&mut ubound_implicit_variables);
                Ok((
                    ArrayDimension {
                        lbound: Some(converted_lbound),
                        ubound: converted_ubound,
                    },
                    lbound_implicit_variables,
                ))
            }
            None => {
                let (converted_ubound, implicit_variables) =
                    self.convert_and_collect_implicit_variables(ubound)?;
                Ok((
                    ArrayDimension {
                        lbound: None,
                        ubound: converted_ubound,
                    },
                    implicit_variables,
                ))
            }
        }
    }
}
