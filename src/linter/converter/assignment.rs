use crate::common::{
    CanCastTo, Locatable, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError,
};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{DimName, DimType, ExpressionNode, Statement};
use crate::parser::{BareName, Name, TypeQualifier};

impl<'a> ConverterImpl<'a> {
    pub fn assignment(
        &mut self,
        name: Name,
        expression_node: crate::parser::ExpressionNode,
    ) -> Result<Statement, QErrorNode> {
        let dim_name = self.resolve_name_in_assignment(name).with_err_no_pos()?;
        let converted_expr: ExpressionNode = self.convert(expression_node)?;
        if converted_expr.can_cast_to(&dim_name) {
            Ok(Statement::Assignment(dim_name, converted_expr))
        } else {
            Err(QError::TypeMismatch).with_err_at(&converted_expr)
        }
    }

    // FIXME still public due to for loop
    pub fn resolve_name_in_assignment(&mut self, name: Name) -> Result<DimName, QError> {
        let bare_name: &BareName = name.as_ref();
        if self.context.is_function_context(bare_name) {
            self.assign_to_function(name)
        } else if self.subs.contains_key(bare_name) {
            // trying to assign to a sub
            Err(QError::DuplicateDefinition)
        } else if !self
            .context
            .resolve_param_assignment(&name, &self.resolver)? // TODO fix me
            && self.functions.contains_key(bare_name)
        {
            // parameter might be hiding a function name so it takes precedence
            Err(QError::DuplicateDefinition)
        } else {
            let dim_name: DimName = self.context.resolve_assignment(&name, &self.resolver)?;
            Ok(dim_name)
        }
    }

    fn demand_function_type<S: AsRef<BareName>>(&self, function_name: S) -> TypeQualifier {
        let Locatable {
            element: (function_type, _),
            ..
        } = self
            .functions
            .get(function_name.as_ref())
            .expect("Function not found");
        *function_type
    }

    fn assign_to_function(&self, name: Name) -> Result<DimName, QError> {
        let function_type: TypeQualifier = self.demand_function_type(&name);
        if name.is_bare_or_of_type(function_type) {
            Ok(DimName::new(name.into(), DimType::BuiltIn(function_type)))
        } else {
            // trying to assign to the function with an explicit wrong type
            Err(QError::DuplicateDefinition)
        }
    }
}
