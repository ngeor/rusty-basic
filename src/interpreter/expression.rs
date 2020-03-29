use super::function_context::FunctionImplementation;
use super::*;
use crate::common::Result;
use crate::parser::*;
use std::io::BufRead;

impl<T: BufRead, S: Stdlib> Interpreter<T, S> {
    pub fn evaluate_expression(&mut self, e: &Expression) -> Result<Variant> {
        match e {
            Expression::IntegerLiteral(i) => Ok(Variant::from(*i)),
            Expression::StringLiteral(s) => Ok(Variant::from(s)),
            Expression::VariableName(qn) => self.get_variable(qn),
            Expression::FunctionCall(name, args) => self._evaluate_function_call(name, args),
            Expression::BinaryExpression(op, left, right) => {
                self._evaluate_binary_expression(op, left, right)
            }
        }
    }

    fn _evaluate_function_call(&mut self, name: &QName, args: &Vec<Expression>) -> Result<Variant> {
        match self
            .function_context
            .get_function_implementation(&name.name())
        {
            Some(function_implementation) => {
                self._do_evaluate_function_call(name, function_implementation, args)
            }
            None => self.err(format!("Function {} not defined", name)),
        }
    }

    fn _do_evaluate_function_call(
        &mut self,
        name: &QName,
        function_implementation: FunctionImplementation,
        args: &Vec<Expression>,
    ) -> Result<Variant> {
        let function_parameters: &Vec<QName> = &function_implementation.parameters;
        if function_parameters.len() != args.len() {
            self.err(format!(
                "Function {} expected {} parameters but {} were given",
                name,
                function_parameters.len(),
                args.len()
            ))
        } else {
            self.push_context()?;
            self._populate_new_context(function_parameters, args)?;
            self.statements(&function_implementation.block)?;
            let result = self.get_variable(name);
            self.pop_context()?;
            result
        }
    }

    fn _populate_new_context(
        &mut self,
        parameter_names: &Vec<QName>,
        arguments: &Vec<Expression>,
    ) -> Result<()> {
        let mut i = 0;
        while i < parameter_names.len() {
            let variable_name = &parameter_names[i];
            let variable_value = self.evaluate_expression(&arguments[i])?;
            self.set_variable(variable_name, variable_value)?;
            i += 1;
        }
        Ok(())
    }

    fn _evaluate_binary_expression(
        &mut self,
        op: &Operand,
        left: &Box<Expression>,
        right: &Box<Expression>,
    ) -> Result<Variant> {
        let left_var: Variant = self.evaluate_expression(left)?;
        let right_var: Variant = self.evaluate_expression(right)?;
        match op {
            Operand::LessOrEqualThan => {
                let cmp = left_var.compare_to(&right_var)?;
                match cmp {
                    std::cmp::Ordering::Less | std::cmp::Ordering::Equal => Ok(V_TRUE),
                    std::cmp::Ordering::Greater => Ok(V_FALSE),
                }
            }
            Operand::Plus => left_var.plus(&right_var),
            Operand::Minus => left_var.minus(&right_var),
            _ => unimplemented!(),
        }
    }
}
