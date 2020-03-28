use crate::common::Result;
use crate::parser::{Parser, Program, TopLevelToken};
use std::io::prelude::*;

mod context;
mod expression;
mod for_loop;
mod function_context;
mod statement;
mod stdlib;
mod sub_call;

use self::context::*;
use self::function_context::FunctionContext;
use self::stdlib::{DefaultStdlib, Stdlib};

pub struct Interpreter<T, S> {
    parser: Parser<T>,
    stdlib: S,
    context_stack: Vec<Context>,
    function_context: FunctionContext,
}

impl<T: BufRead> Interpreter<T, DefaultStdlib> {
    pub fn new(reader: T) -> Interpreter<T, DefaultStdlib> {
        Interpreter::with_stdlib(reader, DefaultStdlib {})
    }
}

impl<T: BufRead, S: Stdlib> Interpreter<T, S> {
    pub fn with_stdlib(reader: T, stdlib: S) -> Interpreter<T, S> {
        Interpreter {
            parser: Parser::new(reader),
            stdlib,
            context_stack: vec![Context::new()],
            function_context: FunctionContext::new(),
        }
    }

    pub fn interpret(&mut self) -> Result<()> {
        let program = self.parser.parse()?;
        self._parse_top_level_declarations(&program)?;
        self._parse_top_level_implementations(&program)?;
        self._search_for_unimplemented_declarations()?;
        self._parse_top_level_statements(&program)
    }

    pub fn push_context(&mut self, c: Context) {
        self.context_stack.push(c);
    }

    pub fn pop_context(&mut self) -> Result<()> {
        if self.context_stack.len() > 1 {
            self.context_stack.remove(self.context_stack.len() - 1);
            Ok(())
        } else {
            Err("Stack underflow".to_string())
        }
    }

    pub fn clone_context(&self) -> Context {
        let stack = &self.context_stack;
        let top_context = &stack[stack.len() - 1];
        top_context.clone()
    }

    fn _parse_top_level_declarations(&mut self, program: &Program) -> Result<()> {
        for top_level_token in program {
            if let TopLevelToken::FunctionDeclaration(f, args) = top_level_token {
                self.function_context.add_function_declaration(&f.name)?;
            }
        }
        Ok(())
    }

    fn _parse_top_level_implementations(&mut self, program: &Program) -> Result<()> {
        for top_level_token in program {
            if let TopLevelToken::FunctionImplementation(f, args, block) = top_level_token {
                let name_args: Vec<&str> = args.iter().map(|x| x.name.as_ref()).collect();
                self.function_context.add_function_implementation(
                    f.name.as_ref(),
                    name_args,
                    block.to_vec(),
                )?;
            }
        }
        Ok(())
    }

    fn _search_for_unimplemented_declarations(&mut self) -> Result<()> {
        for k in self.function_context.get_function_declarations() {
            if let None = self.function_context.get_function_implementation(k) {
                return Err(format!("Function {} is not implemented", k));
            }
        }

        Ok(())
    }

    fn _parse_top_level_statements(&mut self, program: &Program) -> Result<()> {
        for top_level_token in program {
            if let TopLevelToken::Statement(statement) = top_level_token {
                self.statement(statement)?;
            }
        }
        Ok(())
    }
}

impl<T, TStdlib> ReadOnlyContext for Interpreter<T, TStdlib> {
    fn get_variable<S: AsRef<str>>(&self, variable_name: S) -> Result<Variant> {
        self.context_stack[self.context_stack.len() - 1].get_variable(variable_name)
    }
}

impl<T, TStdlib> ReadWriteContext for Interpreter<T, TStdlib> {
    fn set_variable(&mut self, variable_name: String, variable_value: Variant) -> Result<()> {
        let idx = self.context_stack.len() - 1;
        self.context_stack[idx].set_variable(variable_name, variable_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufReader, Cursor};

    pub struct MockStdlib {}

    impl Stdlib for MockStdlib {
        fn print(&self, args: Vec<String>) {
            let mut is_first = true;
            for a in args {
                if is_first {
                    is_first = false;
                } else {
                    print!(" ");
                }
                print!("{}", a)
            }

            println!("")
        }

        fn system(&self) {
            println!("would have exited")
        }

        fn input(&self) -> Result<String> {
            Ok("10".to_string())
        }
    }

    #[test]
    fn test_interpret_print_hello_world() {
        let input = "PRINT \"Hello, world!\"";
        let c = Cursor::new(input);
        let reader = BufReader::new(c);
        let stdlib = MockStdlib {};
        let mut interpreter = Interpreter::with_stdlib(reader, stdlib);
        interpreter.interpret().unwrap();
    }

    fn test_file(filename: &str, stdlib: MockStdlib) {
        let file_path = format!("fixtures/{}", filename);
        let file =
            File::open(file_path).expect(format!("Could not read file {}", filename).as_ref());
        let reader = BufReader::new(file);
        let mut interpreter = Interpreter::with_stdlib(reader, stdlib);
        interpreter.interpret().unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello1() {
        let stdlib = MockStdlib {};
        test_file("HELLO1.BAS", stdlib);
    }

    #[test]
    fn test_interpreter_fixture_hello2() {
        let stdlib = MockStdlib {};
        test_file("HELLO2.BAS", stdlib);
    }

    #[test]
    fn test_interpreter_fixture_hello_s() {
        let stdlib = MockStdlib {};
        test_file("HELLO_S.BAS", stdlib);
    }

    #[test]
    fn test_interpreter_for_print_10() {
        let stdlib = MockStdlib {};
        test_file("FOR_PRINT_10.BAS", stdlib);
    }

    #[test]
    fn test_interpreter_for_nested() {
        let stdlib = MockStdlib {};
        test_file("FOR_NESTED.BAS", stdlib);
    }

    #[test]
    fn test_interpreter_fixture_fib() {
        let stdlib = MockStdlib {};
        test_file("FIB.BAS", stdlib);
    }

    #[test]
    fn test_interpreter_fixture_input() {
        let stdlib = MockStdlib {};
        test_file("INPUT.BAS", stdlib);
    }
}
