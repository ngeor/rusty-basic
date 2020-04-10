use crate::common::{HasLocation, StripLocation};
use crate::parser::*;

mod assignment;
mod casting;
mod context;
mod expression;
mod for_loop;
mod function_call;
mod function_context;
mod if_block;
mod interpreter_error;
mod statement;
mod stdlib;
mod sub_call;
mod variable_setter;
mod variant;

#[cfg(test)]
mod test_utils;

use self::context::*;
use self::function_context::FunctionContext;
pub use self::interpreter_error::*;
pub use self::stdlib::*;
use self::variable_setter::*;
use self::variant::*;

#[derive(Debug)]
pub struct Interpreter<S> {
    stdlib: S,
    context: Option<Context>, // declared as option to be able to use Option.take; it should always have Some value
    function_context: FunctionContext,
}

pub type Result<T> = std::result::Result<T, InterpreterError>;

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn new(stdlib: TStdlib) -> Interpreter<TStdlib> {
        Interpreter {
            stdlib,
            context: Some(Context::new()),
            function_context: FunctionContext::new(),
        }
    }

    pub fn interpret(&mut self, program: ProgramNode) -> Result<()> {
        let mut statements: BlockNode = vec![];
        for top_level_token in program {
            match top_level_token {
                TopLevelTokenNode::FunctionDeclaration(f) => {
                    self.function_context
                        .add_function_declaration(f.resolve(self))?;
                }
                TopLevelTokenNode::FunctionImplementation(f) => {
                    self.function_context
                        .add_function_implementation(f.resolve(self))?;
                }
                TopLevelTokenNode::Statement(s) => {
                    statements.push(s);
                }
            }
        }
        self._search_for_unimplemented_declarations()?;
        self.statements(&statements)
    }

    pub fn push_context(&mut self, result_name: QualifiedName) {
        self.context = self.context.take().map(|x| x.push(result_name));
    }

    pub fn pop_context(&mut self) {
        self.context = self.context.take().map(|x| x.pop());
    }

    fn _search_for_unimplemented_declarations(&mut self) -> Result<()> {
        for name in self.function_context.get_function_declarations() {
            if let None = self.function_context.get_function_implementation(name) {
                return Err(InterpreterError::new_with_pos(
                    "Subprogram not defined",
                    self.function_context
                        .get_function_declaration_pos(name)
                        .unwrap(),
                ));
            }
        }

        Ok(())
    }

    fn context_ref(&self) -> &Context {
        match self.context.as_ref() {
            Some(x) => x,
            None => panic!("Stack underflow"),
        }
    }

    fn context_mut(&mut self) -> &mut Context {
        match self.context.as_mut() {
            Some(x) => x,
            None => panic!("Stack underflow"),
        }
    }
}

impl<TStdlib: Stdlib> TypeResolver for Interpreter<TStdlib> {
    fn resolve(&self, _name: &str) -> TypeQualifier {
        TypeQualifier::BangSingle
    }
}

//
// VariableGetter
//

trait VariableGetter<T> {
    fn get_variable(&self, variable_name: T) -> std::result::Result<&Variant, InterpreterError>;
}

impl<S: Stdlib> VariableGetter<&QualifiedNameNode> for Interpreter<S> {
    fn get_variable(
        &self,
        variable_name: &QualifiedNameNode,
    ) -> std::result::Result<&Variant, InterpreterError> {
        match self.context_ref().get(&variable_name.strip_location()) {
            Some(v) => Ok(v),
            None => Err(InterpreterError::new_with_pos(
                format!("Variable {} not defined", variable_name.strip_location()),
                variable_name.location(),
            )),
        }
    }
}

impl<S: Stdlib> VariableGetter<&NameNode> for Interpreter<S> {
    fn get_variable(
        &self,
        variable_name: &NameNode,
    ) -> std::result::Result<&Variant, InterpreterError> {
        let q: QualifiedNameNode = variable_name.resolve(self);
        self.get_variable(&q)
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;

    #[test]
    fn test_interpret_print_hello_world() {
        let input = "PRINT \"Hello, world!\"";
        let stdlib = MockStdlib::new();
        interpret(input, stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello1() {
        let stdlib = MockStdlib::new();
        interpret_file("HELLO1.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello2() {
        let stdlib = MockStdlib::new();
        interpret_file("HELLO2.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello_s() {
        let stdlib = MockStdlib::new();
        interpret_file("HELLO_S.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_for_print_10() {
        let stdlib = MockStdlib::new();
        interpret_file("FOR_PRINT_10.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_for_nested() {
        let stdlib = MockStdlib::new();
        interpret_file("FOR_NESTED.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_fib_bas() {
        let mut stdlib = MockStdlib::new();
        stdlib.next_input = "10".to_string();
        let interpreter = interpret_file("FIB.BAS", stdlib).unwrap();
        let output = interpreter.stdlib.output;
        assert_eq!(
            output,
            vec![
                "Enter the number of fibonacci to calculate",
                "Fibonacci of 0 is 0",
                "Fibonacci of 1 is 1",
                "Fibonacci of 2 is 1",
                "Fibonacci of 3 is 2",
                "Fibonacci of 4 is 3",
                "Fibonacci of 5 is 5",
                "Fibonacci of 6 is 8",
                "Fibonacci of 7 is 13",
                "Fibonacci of 8 is 21",
                "Fibonacci of 9 is 34",
                "Fibonacci of 10 is 55"
            ]
        );
    }

    #[test]
    fn test_interpreter_fixture_fib_fq_bas() {
        let mut stdlib = MockStdlib::new();
        stdlib.next_input = "11".to_string();
        interpret_file("FIB_FQ.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_input() {
        let stdlib = MockStdlib::new();
        interpret_file("INPUT.BAS", stdlib).unwrap();
    }
}
