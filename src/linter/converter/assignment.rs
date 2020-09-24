use crate::common::{
    CanCastTo, Locatable, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError,
};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{DimName, DimType, ExpressionNode, Statement};
use crate::parser::{BareName, Name};

impl<'a> ConverterImpl<'a> {
    pub fn assignment(
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

    // FIXME still public due to for loop
    pub fn resolve_name_in_assignment(&mut self, name: Name) -> Result<DimName, QError> {
        let bare_name: &BareName = name.as_ref();
        if self.context.is_function_context(&name) {
            // trying to assign to the function
            let Locatable {
                element: (function_type, _),
                ..
            } = self.functions.get(bare_name).unwrap();
            if name.is_bare_or_of_type(*function_type) {
                Ok(DimName::new(
                    bare_name.clone(),
                    DimType::BuiltIn(*function_type),
                ))
            } else {
                // trying to assign to the function with an explicit wrong type
                Err(QError::DuplicateDefinition)
            }
        } else if self.subs.contains_key(bare_name) {
            // trying to assign to a sub
            Err(QError::DuplicateDefinition)
        } else if !self
            .context
            .resolve_param_assignment(&name, &self.resolver)?
            && self.functions.contains_key(bare_name)
        {
            // parameter might be hiding a function name so it takes precedence
            Err(QError::DuplicateDefinition)
        } else {
            let dim_name: DimName = self.context.resolve_assignment(&name, &self.resolver)?;
            Ok(dim_name)
        }
    }
}
