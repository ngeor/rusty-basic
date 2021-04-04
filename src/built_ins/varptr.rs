pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_variable()
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::instruction_generator::{Path, RootPath};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::variant::{QBNumberCast, Variant};

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        match interpreter.context().variables().get_path(0) {
            Some(path) => {
                let address = find_path_address(interpreter, path)?;
                interpreter
                    .context_mut()
                    .set_built_in_function_result(BuiltInFunction::VarPtr, address as i32);
                Ok(())
            }
            _ => {
                Err(QError::VariableRequired) // should not happen
            }
        }
    }

    fn find_path_address<S: InterpreterTrait>(
        interpreter: &S,
        path: &Path,
    ) -> Result<usize, QError> {
        match path {
            Path::ArrayElement(parent_path, indices) => {
                // arrays define a new segment, no need to calculate parent path address
                if let Path::Root(RootPath {
                    name: array_name,
                    shared,
                }) = parent_path.as_ref()
                {
                    let variable_owner = if *shared {
                        interpreter.context().global_variables()
                    } else {
                        interpreter.context().caller_variables()
                    };
                    let array_variable = variable_owner
                        .get_by_name(array_name)
                        .expect("Should have variable");
                    if let Variant::VArray(arr) = array_variable {
                        let indices_as_int: Vec<i32> = indices.try_cast()?;
                        let address = arr.address_offset_of_element(&indices_as_int)?;
                        Ok(address)
                    } else {
                        panic!("Should be an array");
                    }
                } else {
                    // TODO make this enforceable via rust types
                    panic!("Should not be possible to have anything else as parent of an array element")
                }
            }
            Path::Property(parent_path, property_name) => {
                // find address of parent path
                let parent_address = find_path_address(interpreter, parent_path.as_ref())?;
                // calculate offset for given property
                let owner_variable = find_owning_type(interpreter, parent_path.as_ref())?;
                if let Variant::VUserDefined(u) = owner_variable {
                    Ok(parent_address + u.address_offset_of_property(property_name))
                } else {
                    panic!("Should be a property")
                }
            }
            Path::Root(RootPath { name, shared }) => {
                // find address of root path
                let variable_owner = if *shared {
                    interpreter.context().global_variables()
                } else {
                    interpreter.context().caller_variables()
                };
                let size = variable_owner.calculate_var_ptr(name);
                Ok(size)
            }
        }
    }

    fn find_owning_type<'a, S: InterpreterTrait>(
        interpreter: &'a S,
        path: &Path,
    ) -> Result<&'a Variant, QError> {
        match path {
            Path::Root(RootPath { name, shared }) => {
                let variable_owner = if *shared {
                    interpreter.context().global_variables()
                } else {
                    interpreter.context().caller_variables()
                };
                variable_owner
                    .get_by_name(name)
                    .ok_or(QError::VariableRequired)
            }
            Path::Property(parent_path, property_name) => {
                let parent = find_owning_type(interpreter, parent_path.as_ref())?;
                if let Variant::VUserDefined(u) = parent {
                    u.get(property_name).ok_or(QError::ElementNotDefined)
                } else {
                    Err(QError::TypeMismatch)
                }
            }
            Path::ArrayElement(parent_path, indices) => {
                let parent = find_owning_type(interpreter, parent_path.as_ref())?;
                if let Variant::VArray(a) = parent {
                    let int_indices: Vec<i32> = indices.try_cast()?;
                    a.get_element(&int_indices)
                } else {
                    Err(QError::TypeMismatch)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_no_arguments() {
        assert_linter_err!("PRINT VARPTR()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arguments() {
        assert_linter_err!("PRINT VARPTR(A, B)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn test_literal_argument() {
        assert_linter_err!("PRINT VARPTR(3)", QError::VariableRequired);
    }

    #[test]
    fn global_built_in_vars() {
        let input = r#"
        DIM A AS INTEGER
        DIM B AS LONG
        DIM C AS SINGLE
        DIM D AS DOUBLE
        PRINT VARPTR(A)
        PRINT VARPTR(B)
        PRINT VARPTR(C)
        PRINT VARPTR(D)
        "#;
        assert_prints!(input, "0", "2", "6", "10");
    }

    #[test]
    fn inside_sub() {
        let input = r#"
        Hello

        SUB Hello
            DIM A AS INTEGER
            DIM B AS LONG
            PRINT VARPTR(A)
            PRINT VARPTR(B)
        END SUB
        "#;
        assert_prints!(input, "0", "2");
    }

    #[test]
    fn using_shared_variable_inside_sub() {
        let input = r#"
        DIM SHARED C AS SINGLE
        Hello

        SUB Hello
            DIM A AS INTEGER
            DIM B AS LONG
            PRINT VARPTR(A)
            PRINT VARPTR(B)
            PRINT VARPTR(C)
        END SUB
        "#;
        assert_prints!(input, "0", "2", "0");
    }

    #[test]
    fn array_elements_relative_to_array() {
        let input = r#"
        DIM A(1 TO 2)
        PRINT VARPTR(A)
        PRINT VARPTR(A(1))
        PRINT VARPTR(A(2))
        "#;
        assert_prints!(input, "0", "0", "4");
    }

    #[test]
    fn multi_dimensional_array() {
        let input = r#"
        DIM A(1 TO 3, 1 TO 4)
        PRINT VARPTR(A(2, 3))
        "#;
        assert_prints!(input, "24");
    }

    #[test]
    fn property_elements() {
        let input = r#"
        TYPE Card
            Value AS INTEGER
            Suit AS STRING * 5
            Luck AS INTEGER
        END TYPE
        DIM c AS Card
        PRINT VARPTR(c)
        PRINT VARPTR(c.Value)
        PRINT VARPTR(c.Suit)
        PRINT VARPTR(c.Luck)
        "#;
        assert_prints!(input, "0", "0", "2", "7");
    }

    #[test]
    fn nested_property() {
        let input = r#"
        TYPE PostCode
            Digits AS STRING * 4
            Suffix AS STRING * 2
        END TYPE

        TYPE Address
            Street AS STRING * 20
            PostCode AS PostCode
        END TYPE

        DIM A AS Address
        PRINT VARPTR(A.PostCode.Suffix)
        "#;
        assert_prints!(input, "24");
    }

    #[test]
    fn nested_property_on_array_element() {
        let input = r#"
        TYPE PostCode
            Digits AS STRING * 4
            Suffix AS STRING * 2
        END TYPE

        TYPE Address
            Street AS STRING * 20
            PostCode AS PostCode
        END TYPE

        DIM A(1 TO 5) AS Address
        PRINT VARPTR(A(2).PostCode.Suffix)
        "#;
        assert_prints!(input, "50");
    }
}
