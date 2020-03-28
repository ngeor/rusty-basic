use super::context::ReadWriteContext;
use super::context::Variant;
use super::function_context::FunctionImplementation;
use super::*;
use crate::common::Result;
use crate::parser::*;
use std::io::BufRead;

impl<T: BufRead, S: Stdlib> Interpreter<T, S> {
    pub fn evaluate_expression_as_variant(&mut self, e: &Expression) -> Result<Variant> {
        match e {
            Expression::IntegerLiteral(i) => Ok(Variant::VNumber(*i)),
            Expression::StringLiteral(s) => Ok(Variant::VString(s.to_owned())),
            Expression::VariableName(qn) => self._evaluate_variable(qn),
            Expression::FunctionCall(name, args) => self._evaluate_function_call(name, args),
            Expression::BinaryExpression(op, left, right) => self._evaluate_binary_expression(op, left, right),
            _ => Err(format!("Cannot evaluate expression {:?}", e)),
        }
    }

    pub fn evaluate_expression_as_int(&mut self, e: &Expression) -> Result<i32> {
        let v = self.evaluate_expression_as_variant(e)?;
        match v {
            Variant::VNumber(i) => Ok(i),
            Variant::VString(s) => match s.parse::<i32>() {
                Ok(i2) => Ok(i2),
                Err(e) => Err(format!("Could not convert {} to a number: {}", s, e)),
            },
        }
    }

    pub fn evaluate_expression_as_str(&mut self, e: &Expression) -> Result<String> {
        let v = self.evaluate_expression_as_variant(e)?;
        match v {
            Variant::VNumber(i) => Ok(format!("{}", i)),
            Variant::VString(s) => Ok(s),
        }
    }

    fn _evaluate_variable(&self, qn: &NameWithTypeQualifier) -> Result<Variant> {
        self.get_variable(&qn.name)
    }

    fn _evaluate_function_call(
        &mut self,
        name: &String,
        args: &Vec<Expression>,
    ) -> Result<Variant> {
        let function_implementation: FunctionImplementation = self._get_function_implementation(name)?;
        let function_parameters: Vec<String> = function_implementation.parameters;
        if function_parameters.len() != args.len() {
            return Err(format!(
                "Function {} expected {} parameters but {} were given",
                name,
                function_parameters.len(),
                args.len()
            ));
        }

        let new_context: Context = self._populate_new_context(function_parameters, args)?;
        self.push_context(new_context);
        self.statements(&function_implementation.block)?;
        let result = self.get_variable(name);
        self.pop_context()?;
        result
    }

    fn _get_function_implementation(&self, name: &String) -> Result<FunctionImplementation> {
        match self.function_context.get_function_implementation(name) {
            Some(f) => Ok(f),
            None => Err(format!("Could not find function {}", name)),
        }
    }

    fn _populate_new_context(&mut self, parameter_names: Vec<String>, arguments: &Vec<Expression>) -> Result<Context> {
        let mut i = 0;
        let mut new_context: Context = self.clone_context();
        while i < parameter_names.len() {
            let variable_name = parameter_names[i].to_owned();
            let variable_value = self.evaluate_expression_as_variant(&arguments[i])?;
            new_context.set_variable(variable_name, variable_value)?;
            i += 1;
        }
        Ok(new_context)
    }

    fn _evaluate_binary_expression(&mut self, op: &Operand, left: &Box<Expression>, right: &Box<Expression>) -> Result<Variant> {
        let left_var: Variant = self.evaluate_expression_as_variant(left)?;
        let right_var: Variant = self.evaluate_expression_as_variant(right)?;
        match op {
            Operand::LessOrEqualThan => {
                let cmp = left_var.compare_to(&right_var)?;
                match cmp {
                    std::cmp::Ordering::Less | std::cmp::Ordering::Equal => Ok(Variant::VNumber(1)),
                    std::cmp::Ordering::Greater => Ok(Variant::VNumber(0))
                }
            },
            Operand::Plus => Ok(left_var.plus(&right_var)),
            Operand::Minus => left_var.minus(&right_var),
            _ => unimplemented!()
        }
    }
}
