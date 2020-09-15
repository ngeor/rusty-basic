use super::expression_reducer::ExpressionReducer;
use super::post_conversion_linter::PostConversionLinter;
use super::subprogram_context::{FunctionMap, SubMap};
use super::types::*;
use crate::common::*;
use crate::linter::converter;
use crate::parser;
use crate::parser::{BareName, BareNameNode, QualifiedName, QualifiedNameNode};
use std::collections::HashSet;

pub fn lint(
    program: parser::ProgramNode,
) -> Result<(ProgramNode, ResolvedUserDefinedTypes), QErrorNode> {
    // convert to fully typed
    let (result, functions, subs, user_defined_types, names_without_dot) =
        converter::convert(program)?;
    // lint
    apply_linters(&result, &functions, &subs, &names_without_dot)?;
    // reduce
    let reducer = super::undefined_function_reducer::UndefinedFunctionReducer {
        functions: &functions,
    };
    reducer
        .visit_program(result)
        .map(|p| (p, user_defined_types))
}

fn apply_linters(
    result: &ProgramNode,
    functions: &FunctionMap,
    subs: &SubMap,
    names_without_dot: &HashSet<CaseInsensitiveString>,
) -> Result<(), QErrorNode> {
    let linter = super::for_next_counter_match::ForNextCounterMatch {};
    linter.visit_program(&result)?;

    let linter = NoDotNames { names_without_dot };
    linter.visit_program(&result)?;

    let linter = super::built_in_linter::BuiltInLinter {};
    linter.visit_program(&result)?;

    let linter = super::user_defined_function_linter::UserDefinedFunctionLinter { functions };
    linter.visit_program(&result)?;

    let linter = super::user_defined_sub_linter::UserDefinedSubLinter { subs };
    linter.visit_program(&result)?;

    let linter = super::select_case_linter::SelectCaseLinter {};
    linter.visit_program(&result)?;

    let mut linter = super::label_linter::LabelLinter::new();
    linter.visit_program(&result)?;
    linter.switch_to_validating_mode();
    linter.visit_program(&result)?;
    Ok(())
}

// ========================================================
// NoDotNames
// ========================================================

pub struct NoDotNames<'a> {
    pub names_without_dot: &'a HashSet<CaseInsensitiveString>,
}

trait NoDotNamesCheck<T, E> {
    fn ensure_no_dots(&self, x: &T) -> Result<(), E>;
}

impl<'a> NoDotNamesCheck<FunctionImplementation, QErrorNode> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &FunctionImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(&x.name)?;
        self.ensure_no_dots(&x.params)
    }
}

impl<'a> NoDotNamesCheck<SubImplementation, QErrorNode> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &SubImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(&x.name)?;
        self.ensure_no_dots(&x.params)
    }
}

impl<'a> NoDotNamesCheck<ResolvedDeclaredNameNodes, QErrorNode> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &ResolvedDeclaredNameNodes) -> Result<(), QErrorNode> {
        x.into_iter().map(|x| self.ensure_no_dots(x)).collect()
    }
}

impl<'a> NoDotNamesCheck<ResolvedDeclaredNameNode, QErrorNode> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &ResolvedDeclaredNameNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl<'a> NoDotNamesCheck<ResolvedDeclaredName, QError> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &ResolvedDeclaredName) -> Result<(), QError> {
        let bare_name: &BareName = x.as_ref();
        self.ensure_no_dots(bare_name)
    }
}

impl<'a> NoDotNamesCheck<QualifiedNameNode, QErrorNode> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &QualifiedNameNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl<'a> NoDotNamesCheck<BareNameNode, QErrorNode> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &BareNameNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).with_err_at(pos)
    }
}

impl<'a> NoDotNamesCheck<QualifiedName, QError> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &QualifiedName) -> Result<(), QError> {
        let QualifiedName { name, .. } = x;
        self.ensure_no_dots(name)
    }
}

impl<'a> NoDotNamesCheck<BareName, QError> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &BareName) -> Result<(), QError> {
        match x.prefix('.') {
            Some(first) => {
                if self.names_without_dot.contains(&first) {
                    Err(QError::DotClash)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

impl<'a> NoDotNamesCheck<Vec<ExpressionNode>, QErrorNode> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        x.into_iter().map(|x| self.ensure_no_dots(x)).collect()
    }
}

impl<'a> NoDotNamesCheck<ExpressionNode, QErrorNode> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &ExpressionNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = x;
        self.ensure_no_dots(element).patch_err_pos(pos)
    }
}

impl<'a> NoDotNamesCheck<Expression, QErrorNode> for NoDotNames<'a> {
    fn ensure_no_dots(&self, x: &Expression) -> Result<(), QErrorNode> {
        match x {
            Expression::Constant(qualified_name) => {
                self.ensure_no_dots(qualified_name).with_err_no_pos()
            }
            Expression::Variable(resolved_declared_name) => self
                .ensure_no_dots(resolved_declared_name)
                .with_err_no_pos(),
            Expression::FunctionCall(name, args) => {
                self.ensure_no_dots(name).with_err_no_pos()?;
                self.ensure_no_dots(args)
            }
            Expression::BuiltInFunctionCall(_, args) => self.ensure_no_dots(args),
            Expression::BinaryExpression(_, left, right) => {
                self.ensure_no_dots(left.as_ref())?;
                self.ensure_no_dots(right.as_ref())
            }
            Expression::UnaryExpression(_, child) | Expression::Parenthesis(child) => {
                self.ensure_no_dots(child.as_ref())
            }
            _ => Ok(()),
        }
    }
}

impl<'a> PostConversionLinter for NoDotNames<'a> {
    fn visit_function_implementation(&self, f: &FunctionImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(f)?;
        self.visit_statement_nodes(&f.body)
    }

    fn visit_sub_implementation(&self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.ensure_no_dots(s)?;
        self.visit_statement_nodes(&s.body)
    }

    fn visit_dim(&self, d: &ResolvedDeclaredNameNode) -> Result<(), QErrorNode> {
        self.ensure_no_dots(d)
    }

    fn visit_assignment(
        &self,
        name: &ResolvedDeclaredName,
        v: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        self.ensure_no_dots(name).with_err_no_pos()?;
        self.visit_expression(v)
    }

    fn visit_for_loop(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        // TODO verify variable name

        self.visit_expression(&f.lower_bound)?;
        self.visit_expression(&f.upper_bound)?;
        match &f.step {
            Some(s) => self.visit_expression(s)?,
            None => (),
        }
        self.visit_statement_nodes(&f.statements)
    }

    fn visit_const(
        &self,
        left: &QualifiedNameNode,
        right: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        self.ensure_no_dots(left)?;
        self.visit_expression(right)
    }

    fn visit_expression(&self, e: &ExpressionNode) -> Result<(), QErrorNode> {
        self.ensure_no_dots(e)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::linter::test_utils::*;
    use crate::linter::*;
    use crate::parser::Operator;

    mod assignment {
        use super::*;

        #[test]
        fn name_clashes_with_other_sub_name() {
            let program = r#"
            SUB Hello
            END SUB
            SUB Oops
            Hello = 2
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 5, 13);
        }

        #[test]
        fn literals_type_mismatch() {
            assert_linter_err!("X = \"hello\"", QError::TypeMismatch, 1, 5);
            assert_linter_err!("X! = \"hello\"", QError::TypeMismatch, 1, 6);
            assert_linter_err!("X# = \"hello\"", QError::TypeMismatch, 1, 6);
            assert_linter_err!("A$ = 1.0", QError::TypeMismatch, 1, 6);
            assert_linter_err!("A$ = 1", QError::TypeMismatch, 1, 6);
            assert_linter_err!("A$ = -1", QError::TypeMismatch, 1, 6);
            assert_linter_err!("X% = \"hello\"", QError::TypeMismatch, 1, 6);
            assert_linter_err!("X& = \"hello\"", QError::TypeMismatch, 1, 6);
        }

        #[test]
        fn assign_to_const() {
            let program = "
            CONST X = 3.14
            X = 6.28
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 13);
        }

        #[test]
        fn assign_to_parent_const() {
            let program = r#"
            CONST X = 42
            SUB Hello
            X = 3
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
        }

        #[test]
        fn assign_integer_to_extended_string() {
            let program = r#"
            X = 1
            IF X = 0 THEN DIM A AS STRING
            A = 42
            "#;
            assert_linter_err!(program, QError::TypeMismatch, 4, 17);
        }

        #[test]
        fn test_assign_binary_plus() {
            assert_eq!(
                linter_ok("X% = 1 + 2.1"),
                vec![TopLevelToken::Statement(Statement::Assignment(
                    ResolvedDeclaredName::parse("X%"),
                    Expression::BinaryExpression(
                        Operator::Plus,
                        Box::new(Expression::IntegerLiteral(1).at_rc(1, 6),),
                        Box::new(Expression::SingleLiteral(2.1).at_rc(1, 10))
                    )
                    .at_rc(1, 8)
                ))
                .at_rc(1, 1)]
            );
        }
    }

    mod constant {
        use super::*;

        #[test]
        fn function_call_not_allowed() {
            let program = r#"
            CONST X = Add(1, 2)
            "#;
            assert_linter_err!(program, QError::InvalidConstant, 2, 23);
        }

        #[test]
        fn variable_not_allowed() {
            let program = r#"
            X = 42
            CONST A = X + 1
            "#;
            assert_linter_err!(program, QError::InvalidConstant, 3, 23);
        }

        #[test]
        fn variable_already_exists() {
            let program = "
            X = 42
            CONST X = 32
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn variable_already_exists_as_sub_call_param() {
            let program = "
            INPUT X%
            CONST X = 1
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn const_already_exists() {
            let program = "
            CONST X = 32
            CONST X = 33
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn qualified_usage_from_string_literal() {
            let program = r#"
            CONST X! = "hello"
            "#;
            assert_linter_err!(program, QError::TypeMismatch, 2, 24);
        }

        #[test]
        fn const_after_dim_duplicate_definition() {
            let program = r#"
            DIM A AS STRING
            CONST A = "hello"
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn test_global_const_cannot_have_function_name() {
            let program = r#"
            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            CONST GetAction = 42
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 5, 19);
        }

        #[test]
        fn test_local_const_cannot_have_function_name() {
            let program = r#"
            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            FUNCTION Echo(X)
                CONST GetAction = 42
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 6, 23);
        }

        #[test]
        fn test_forward_const_not_allowed() {
            let input = "
            CONST A = B + 1
            CONST B = 42";
            assert_linter_err!(input, QError::InvalidConstant, 2, 23);
        }
    }

    /// Three step tests:
    /// 1. DIM a new variable
    /// 2. Assign to the variable
    /// 3. Use it in an expression
    mod dim_assign_use_in_expr {
        use super::*;

        #[test]
        fn bare() {
            let program = r#"
            DIM A
            A = 42
            PRINT A
            "#;
            assert_eq!(
                linter_ok(program),
                vec![
                    TopLevelToken::Statement(Statement::Dim(
                        ResolvedDeclaredName::parse("A!").at_rc(2, 17)
                    ))
                    .at_rc(2, 13),
                    TopLevelToken::Statement(Statement::Assignment(
                        ResolvedDeclaredName::parse("A!"),
                        Expression::IntegerLiteral(42).at_rc(3, 17)
                    ))
                    .at_rc(3, 13),
                    TopLevelToken::Statement(Statement::BuiltInSubCall(
                        BuiltInSub::Print,
                        vec![Expression::Variable(ResolvedDeclaredName::parse("A!")).at_rc(4, 19)]
                    ))
                    .at_rc(4, 13)
                ]
            );
        }

        #[test]
        fn compact_string() {
            let program = r#"
            DIM A$
            A$ = "hello"
            PRINT A$
            "#;
            assert_eq!(
                linter_ok(program),
                vec![
                    TopLevelToken::Statement(Statement::Dim(
                        ResolvedDeclaredName::parse("A$").at_rc(2, 17)
                    ))
                    .at_rc(2, 13),
                    TopLevelToken::Statement(Statement::Assignment(
                        ResolvedDeclaredName::parse("A$"),
                        Expression::StringLiteral("hello".to_string()).at_rc(3, 18)
                    ))
                    .at_rc(3, 13),
                    TopLevelToken::Statement(Statement::BuiltInSubCall(
                        BuiltInSub::Print,
                        vec![Expression::Variable(ResolvedDeclaredName::parse("A$")).at_rc(4, 19)]
                    ))
                    .at_rc(4, 13)
                ]
            );
        }

        #[test]
        fn extended_string() {
            let program = r#"
            DIM A AS STRING
            A = "hello"
            PRINT A
            "#;
            assert_eq!(
                linter_ok(program),
                vec![
                    TopLevelToken::Statement(Statement::Dim(
                        ResolvedDeclaredName::parse("A$").at_rc(2, 17)
                    ))
                    .at_rc(2, 13),
                    TopLevelToken::Statement(Statement::Assignment(
                        ResolvedDeclaredName::parse("A$"),
                        Expression::StringLiteral("hello".to_string()).at_rc(3, 17)
                    ))
                    .at_rc(3, 13),
                    TopLevelToken::Statement(Statement::BuiltInSubCall(
                        BuiltInSub::Print,
                        vec![Expression::Variable(ResolvedDeclaredName::parse("A$")).at_rc(4, 19)]
                    ))
                    .at_rc(4, 13)
                ]
            );
        }

        #[test]
        fn user_defined_type() {
            let input = r#"
            TYPE Card
                Value AS INTEGER
                Suit AS STRING * 9
            END TYPE
            DIM A AS Card
            DIM B AS Card
            A = B
            "#;
            let (program, user_defined_types) = linter_ok_with_types(input);
            assert_eq!(
                program,
                vec![
                    TopLevelToken::Statement(Statement::Dim(
                        ResolvedDeclaredName::user_defined("A", "Card").at_rc(6, 17)
                    ))
                    .at_rc(6, 13),
                    TopLevelToken::Statement(Statement::Dim(
                        ResolvedDeclaredName::user_defined("B", "Card").at_rc(7, 17)
                    ))
                    .at_rc(7, 13),
                    TopLevelToken::Statement(Statement::Assignment(
                        ResolvedDeclaredName::user_defined("A", "Card"),
                        Expression::Variable(ResolvedDeclaredName::user_defined("B", "Card"))
                            .at_rc(8, 17)
                    ))
                    .at_rc(8, 13)
                ]
            );
            assert_eq!(
                user_defined_types.len(),
                1,
                "Expected one user defined type"
            );
            assert!(
                user_defined_types.contains_key(&"Card".into()),
                "Expected to contain the `Card` type"
            );
            assert_eq!(
                *user_defined_types.get(&"Card".into()).unwrap(),
                ResolvedUserDefinedType {
                    name: "Card".into(),
                    elements: vec![
                        ResolvedElement {
                            name: "Value".into(),
                            element_type: ResolvedElementType::Integer
                        },
                        ResolvedElement {
                            name: "Suit".into(),
                            element_type: ResolvedElementType::String(9)
                        },
                    ]
                }
            );
        }

        #[test]
        fn user_defined_type_integer_element() {
            let input = r#"
            TYPE Card
                Value AS INTEGER
                Suit AS STRING * 9
            END TYPE
            DIM A AS Card
            A.Value = 42
            PRINT A.Value
            "#;
            assert_eq!(
                linter_ok(input),
                vec![
                    TopLevelToken::Statement(Statement::Dim(
                        ResolvedDeclaredName::user_defined("A", "Card").at_rc(6, 17)
                    ))
                    .at_rc(6, 13),
                    TopLevelToken::Statement(Statement::Assignment(
                        ResolvedDeclaredName::Many(
                            UserDefinedName {
                                name: "A".into(),
                                type_name: "Card".into()
                            },
                            Members::Leaf {
                                name: "Value".into(),
                                element_type: ResolvedElementType::Integer
                            }
                        ),
                        Expression::IntegerLiteral(42).at_rc(7, 23)
                    ))
                    .at_rc(7, 13),
                    TopLevelToken::Statement(Statement::BuiltInSubCall(
                        BuiltInSub::Print,
                        vec![Expression::Variable(ResolvedDeclaredName::Many(
                            UserDefinedName {
                                name: "A".into(),
                                type_name: "Card".into()
                            },
                            Members::Leaf {
                                name: "Value".into(),
                                element_type: ResolvedElementType::Integer
                            }
                        ))
                        .at_rc(8, 19)]
                    ))
                    .at_rc(8, 13)
                ]
            );
        }

        #[test]
        fn user_defined_type_string_element() {
            let input = r#"
            TYPE Card
                Value AS INTEGER
                Suit AS STRING * 9
            END TYPE
            DIM A AS Card
            A.Suit = "diamonds"
            PRINT A.Suit
            "#;
            assert_eq!(
                linter_ok(input),
                vec![
                    TopLevelToken::Statement(Statement::Dim(
                        ResolvedDeclaredName::user_defined("A", "Card").at_rc(6, 17)
                    ))
                    .at_rc(6, 13),
                    TopLevelToken::Statement(Statement::Assignment(
                        ResolvedDeclaredName::Many(
                            UserDefinedName {
                                name: "A".into(),
                                type_name: "Card".into()
                            },
                            Members::Leaf {
                                name: "Suit".into(),
                                element_type: ResolvedElementType::String(9)
                            }
                        ),
                        Expression::StringLiteral("diamonds".to_owned()).at_rc(7, 22)
                    ))
                    .at_rc(7, 13),
                    TopLevelToken::Statement(Statement::BuiltInSubCall(
                        BuiltInSub::Print,
                        vec![Expression::Variable(ResolvedDeclaredName::Many(
                            UserDefinedName {
                                name: "A".into(),
                                type_name: "Card".into()
                            },
                            Members::Leaf {
                                name: "Suit".into(),
                                element_type: ResolvedElementType::String(9)
                            }
                        ))
                        .at_rc(8, 19)]
                    ))
                    .at_rc(8, 13)
                ]
            );
        }
    }

    mod dim {
        use super::*;

        #[test]
        fn test_dim_duplicate_definition_same_builtin_type() {
            let program = r#"
            DIM A AS STRING
            DIM A AS STRING
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_duplicate_definition_different_builtin_type() {
            let program = r#"
            DIM A AS STRING
            DIM A AS INTEGER
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_after_const_duplicate_definition() {
            let program = r#"
            CONST A = "hello"
            DIM A AS STRING
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_after_variable_assignment_duplicate_definition() {
            let program = r#"
            A = 42
            DIM A AS INTEGER
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_string_duplicate_definition() {
            let program = r#"
            DIM A$
            DIM A$
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_bare_duplicate_definition() {
            let program = r#"
            DIM A
            DIM A
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_single_bare_duplicate_definition() {
            // single is the default type
            let program = r#"
            DIM A!
            DIM A
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_bare_single_duplicate_definition() {
            // single is the default type
            let program = r#"
            DIM A
            DIM A!
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_compact_bare_integer_duplicate_definition() {
            let program = r#"
            DEFINT A-Z
            DIM A
            DIM A%
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 17);
        }

        #[test]
        fn test_dim_extended_inside_sub_name_clashing_sub_name() {
            let program = r#"
            SUB Hello
            Dim Hello AS STRING
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_bare_inside_sub_name_clashing_other_sub_name() {
            let program = r#"
            SUB Oops
            END SUB

            SUB Hello
            Dim Oops
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 6, 17);
        }

        #[test]
        fn test_dim_extended_inside_sub_name_clashing_param_name() {
            let program = r#"
            SUB Hello(Oops)
            Dim Oops AS STRING
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_extended_inside_function_name_clashing_function_name() {
            let program = r#"
            FUNCTION Hello
            Dim Hello AS STRING
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }

        #[test]
        fn test_dim_extended_inside_function_name_clashing_other_function_name() {
            let program = r#"
            FUNCTION Hello
            Dim Bar AS STRING
            END FUNCTION
            FUNCTION Bar
            END Function
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
        }
    }

    mod function_implementation {
        use super::*;

        #[test]
        fn test_function_param_clashing_sub_name_declared_earlier() {
            let program = r#"
            SUB Hello
            END SUB

            FUNCTION Adding(Hello)
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 5, 29);
        }

        #[test]
        fn test_function_param_clashing_sub_name_declared_later() {
            let program = r#"
            FUNCTION Adding(Hello)
            END FUNCTION

            SUB Hello
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 29);
        }

        #[test]
        fn test_function_param_of_different_type_clashing_function_name() {
            let program = r#"
            FUNCTION Adding(Adding$)
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 29);
        }

        #[test]
        fn test_function_param_clashing_function_name_extended_same_type() {
            let program = r#"
            FUNCTION Adding(Adding AS SINGLE)
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 29);
        }

        #[test]
        fn test_function_param_duplicate() {
            let program = r#"
            FUNCTION Adding(Adding, Adding)
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 37);
        }

        #[test]
        fn test_no_args_function_call_cannot_assign_to_variable() {
            let program = r#"
            DECLARE FUNCTION GetAction$

            GetAction% = 42

            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
        }

        #[test]
        fn test_function_call_without_implementation() {
            let program = "
            DECLARE FUNCTION Add(A, B)
            X = Add(1, 2)
            ";
            assert_linter_err!(program, QError::SubprogramNotDefined, 2, 13);
        }

        #[test]
        fn test_cannot_override_built_in_function_with_declaration() {
            let program = r#"
            DECLARE FUNCTION Environ$
            PRINT "Hello"
            FUNCTION Environ$
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
        }

        #[test]
        fn test_cannot_override_built_in_function_without_declaration() {
            let program = r#"
            PRINT "Hello"
            FUNCTION Environ$
            END FUNCTION
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 13);
        }

        #[test]
        fn test_cannot_call_built_in_function_with_wrong_type() {
            let program = r#"
            PRINT "Hello", Environ%("oops")
            "#;
            assert_linter_err!(program, QError::TypeMismatch, 2, 28);
        }

        #[test]
        fn test_function_call_missing_with_string_arguments_gives_type_mismatch() {
            let program = "
            X = Add(\"1\", \"2\")
            ";
            assert_linter_err!(program, QError::ArgumentTypeMismatch, 2, 21);
        }

        #[test]
        fn test_function_dotted_name_clashes_variable_of_user_defined_type() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            DIM A AS Card

            FUNCTION A.B
            END FUNCTION
            ";
            assert_linter_err!(input, QError::DotClash, 8, 22);
        }

        #[test]
        fn test_function_dotted_name_clashes_variable_of_user_defined_type_in_other_function() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            FUNCTION A.B
            END FUNCTION

            FUNCTION C.D
                DIM A AS Card
            END FUNCTION
            ";
            assert_linter_err!(input, QError::DotClash, 6, 22);
        }

        #[test]
        fn test_function_dotted_name_clashes_variable_of_user_defined_type_in_other_function_following(
        ) {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            FUNCTION A.B
                DIM C AS Card
            END FUNCTION

            FUNCTION C.D
            END FUNCTION
            ";
            assert_linter_err!(input, QError::DotClash, 10, 22);
        }
    }

    mod sub_implementation {
        use super::*;

        #[test]
        fn test_sub_param_clashing_sub_name() {
            let program = r#"
            SUB Hello(Hello)
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 23);
        }

        #[test]
        fn test_sub_param_clashing_other_sub_name_declared_earlier() {
            let program = r#"
            SUB Hello
            END SUB
            SUB Goodbye(Hello)
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 25);
        }

        #[test]
        fn test_sub_param_clashing_other_sub_name_declared_later() {
            let program = r#"
            SUB Goodbye(Hello)
            END SUB
            SUB Hello
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 25);
        }

        #[test]
        fn test_sub_param_duplicate() {
            let program = r#"
            SUB Hello(A, A)
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 26);
        }

        #[test]
        fn test_sub_param_extended_duplicate() {
            let program = r#"
            SUB Hello(A AS INTEGER, A AS STRING)
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 2, 37);
        }

        #[test]
        fn test_cannot_override_built_in_sub_with_declaration() {
            let program = r#"
            DECLARE SUB Environ
            PRINT "Hello"
            SUB Environ
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
        }

        #[test]
        fn test_cannot_override_built_in_sub_without_declaration() {
            let program = r#"
            PRINT "Hello"
            SUB Environ
            END SUB
            "#;
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 13);
        }

        #[test]
        fn test_by_ref_parameter_type_mismatch() {
            let program = "
            DECLARE SUB Hello(N)
            A% = 42
            Hello A%
            SUB Hello(N)
                N = N + 1
            END SUB
            ";
            assert_linter_err!(program, QError::ArgumentTypeMismatch, 4, 19);
        }

        #[test]
        fn test_by_ref_parameter_type_mismatch_user_defined_type() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            DIM c AS Card
            Test c.Value

            SUB Test(N)
            END SUB
            ";
            assert_linter_err!(input, QError::ArgumentTypeMismatch, 7, 18);
        }

        #[test]
        fn test_sub_user_defined_param_cannot_contain_dot() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB Test(A.B AS Card)
            END SUB
            ";
            // QBasic actually reports the error on the dot
            assert_linter_err!(input, QError::IdentifierCannotIncludePeriod, 6, 22);
        }

        #[test]
        fn test_sub_dotted_name_clashes_variable_of_user_defined_type() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            DIM A AS Card

            SUB A.B
            END SUB
            ";
            // QBasic actually reports the error on the dot
            assert_linter_err!(input, QError::DotClash, 8, 17);
        }

        #[test]
        fn test_sub_dotted_name_clashes_variable_of_user_defined_type_in_other_sub() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B
            END SUB

            SUB C.D
                DIM A AS Card
            END SUB
            ";
            assert_linter_err!(input, QError::DotClash, 6, 17);
        }

        #[test]
        fn test_sub_dotted_name_clashes_variable_of_user_defined_type_in_other_sub_following() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B
                DIM C AS Card
            END SUB

            SUB C.D
            END SUB
            ";
            assert_linter_err!(input, QError::DotClash, 10, 17);
        }

        #[test]
        fn test_sub_param_dotted_name_clashes_variable_of_user_defined_type_in_other_sub_following()
        {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B
                DIM C AS Card
            END SUB

            SUB Oops(C.D AS INTEGER)
            END SUB
            ";
            assert_linter_err!(input, QError::DotClash, 10, 22);
        }

        #[test]
        fn test_sub_param_dotted_name_clashes_param_of_user_defined_type_in_other_sub_following() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B(C AS Card)
            END SUB

            SUB Oops(C.D AS INTEGER)
            END SUB
            ";
            assert_linter_err!(input, QError::DotClash, 9, 22);
        }

        #[test]
        fn test_sub_variable_dotted_name_clashes_variable_of_user_defined_type_in_other_sub() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B
                DIM X AS Card
            END SUB

            SUB Oops
                DIM X.Y AS INTEGER
            END SUB
            ";
            assert_linter_err!(input, QError::DotClash, 11, 21);
        }
    }

    mod select_case {
        use super::*;

        #[test]
        fn test_select_wrong_type_in_simple_case() {
            let input = r#"
        SELECT CASE 42
            CASE "book"
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 18);
        }

        #[test]
        fn test_select_wrong_type_in_range_case_upper() {
            let input = r#"
        SELECT CASE 42
            CASE 1 TO "book"
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 23);
        }

        #[test]
        fn test_select_wrong_type_in_range_case_lower() {
            let input = r#"
        SELECT CASE 42
            CASE "abc" TO 12
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 18);
        }

        #[test]
        fn test_select_wrong_type_in_range_case_both() {
            let input = r#"
        SELECT CASE 42
            CASE "abc" TO "def"
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 18);
        }

        #[test]
        fn test_select_wrong_type_in_is() {
            let input = r#"
        SELECT CASE 42
            CASE IS >= "abc"
                PRINT "hi"
        END SELECT
        "#;
            assert_linter_err!(input, QError::TypeMismatch, 3, 24);
        }
    }

    mod go_to {
        use super::*;

        #[test]
        fn on_error_go_to_missing_label() {
            let input = r#"
            ON ERROR GOTO ErrTrap
            "#;
            assert_linter_err!(input, QError::LabelNotDefined, 2, 13);
        }

        #[test]
        fn go_to_missing_label() {
            let input = "
            GOTO Jump
            ";
            assert_linter_err!(input, QError::LabelNotDefined, 2, 13);
        }

        #[test]
        fn go_to_duplicate_label() {
            let input = "
            GOTO Jump
            Jump:
            Jump:
            ";
            assert_linter_err!(input, QError::DuplicateLabel, 4, 13);
        }
    }

    mod for_loop {
        use super::*;

        #[test]
        fn test_for_loop_with_wrong_next_counter() {
            let input = "
            FOR i% = 1 TO 5
                PRINT i%
            NEXT i
            ";
            assert_linter_err!(input, QError::NextWithoutFor, 4, 18);
        }
    }

    mod expression {
        use super::*;

        macro_rules! assert_condition_err {
            ($condition:expr, $col:expr) => {
                let program = format!(
                    "
                IF {} THEN
                    PRINT \"hi\"
                END IF
                ",
                    $condition
                );
                assert_linter_err!(program, QError::TypeMismatch, 2, $col);
            };
        }

        #[test]
        fn test_type_mismatch() {
            assert_linter_err!("X = 1.1 + \"hello\"", QError::TypeMismatch, 1, 11);
            assert_linter_err!("X = 1.1# + \"hello\"", QError::TypeMismatch, 1, 12);
            assert_linter_err!("X$ = \"hello\" + 1", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" + 1.1", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" + 1.1#", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X% = 1 + \"hello\"", QError::TypeMismatch, 1, 10);
            assert_linter_err!("X& = 1 + \"hello\"", QError::TypeMismatch, 1, 10);
            assert_linter_err!("X = 1.1 - \"hello\"", QError::TypeMismatch, 1, 11);
            assert_linter_err!("X = 1.1# - \"hello\"", QError::TypeMismatch, 1, 12);
            assert_linter_err!("X$ = \"hello\" - \"hi\"", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" - 1", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" - 1.1", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = \"hello\" - 1.1#", QError::TypeMismatch, 1, 16);
            assert_linter_err!("X$ = 1 - \"hello\"", QError::TypeMismatch, 1, 10);
            assert_linter_err!("X& = 1 - \"hello\"", QError::TypeMismatch, 1, 10);
            assert_linter_err!(r#"PRINT "hello" * 5"#, QError::TypeMismatch, 1, 17);
            assert_linter_err!(r#"PRINT "hello" / 5"#, QError::TypeMismatch, 1, 17);
            assert_linter_err!("X = -\"hello\"", QError::TypeMismatch, 1, 6);
            assert_linter_err!("X% = -\"hello\"", QError::TypeMismatch, 1, 7);
            assert_linter_err!("X = NOT \"hello\"", QError::TypeMismatch, 1, 9);
            assert_linter_err!("X% = NOT \"hello\"", QError::TypeMismatch, 1, 10);

            assert_linter_err!(r#"PRINT 1 AND "hello""#, QError::TypeMismatch, 1, 13);
            assert_linter_err!(r#"PRINT "hello" AND 1"#, QError::TypeMismatch, 1, 19);
            assert_linter_err!(r#"PRINT "hello" AND "bye""#, QError::TypeMismatch, 1, 19);

            assert_linter_err!(r#"PRINT 1 AND #1"#, QError::TypeMismatch, 1, 13);
            assert_linter_err!(r#"PRINT #1 AND 1"#, QError::TypeMismatch, 1, 14);
            assert_linter_err!(r#"PRINT #1 AND #1"#, QError::TypeMismatch, 1, 14);
        }

        #[test]
        fn test_condition_type_mismatch() {
            assert_condition_err!("9.1 < \"hello\"", 26);
            assert_condition_err!("9.1# < \"hello\"", 27);
            assert_condition_err!("\"hello\" < 3.14", 30);
            assert_condition_err!("\"hello\" < 3", 30);
            assert_condition_err!("\"hello\" < 3.14#", 30);
            assert_condition_err!("9 < \"hello\"", 24);
            assert_condition_err!("9.1 <= \"hello\"", 27);
            assert_condition_err!("9.1# <= \"hello\"", 28);
            assert_condition_err!("\"hello\" <= 3.14", 31);
            assert_condition_err!("\"hello\" <= 3", 31);
            assert_condition_err!("\"hello\" <= 3.14#", 31);
            assert_condition_err!("9 <= \"hello\"", 25);
        }

        #[test]
        fn qualified_const_usage_wrong_type() {
            let program = "
            CONST X = 42
            PRINT X!
            ";
            assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
        }
    }

    mod user_defined_type {
        use super::*;

        #[test]
        fn duplicate_type_throws_duplicate_definition() {
            let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            TYPE Card
                Value AS INTEGER
            END TYPE";
            assert_linter_err!(input, QError::DuplicateDefinition, 6, 13);
        }

        #[test]
        fn duplicate_element_name() {
            let input = "
            TYPE Card
                Value AS INTEGER
                Value AS INTEGER
            END TYPE
            ";
            assert_linter_err!(input, QError::DuplicateDefinition, 4, 17);
        }

        #[test]
        fn element_using_container_type_throws_type_not_defined() {
            let input = "
            TYPE Card
                Item AS Card
            END TYPE";
            // TODO QBasic actually positions the error on the "AS" keyword
            assert_linter_err!(input, QError::TypeNotDefined, 3, 25);
        }

        #[test]
        fn dim_using_undefined_type() {
            let input = "DIM X AS Card";
            // TODO QBasic actually positions the error on the "AS" keyword
            assert_linter_err!(input, QError::TypeNotDefined, 1, 5);
        }

        #[test]
        fn using_type_before_defined_throws_type_not_defined() {
            let input = "
            TYPE Address
                PostCode AS PostCode
            END TYPE

            TYPE PostCode
                Prefix AS INTEGER
                Suffix AS STRING * 2
            END TYPE";
            assert_linter_err!(input, QError::TypeNotDefined, 3, 29);
        }

        #[test]
        fn string_length_must_be_constant() {
            let input = "
            TYPE Invalid
                N AS STRING * A
            END TYPE";
            assert_linter_err!(input, QError::InvalidConstant, 3, 31);
        }

        #[test]
        fn string_length_must_be_constant_const_cannot_follow_type() {
            let input = "
            TYPE Invalid
                N AS STRING * A
            END TYPE

            CONST A = 10";
            assert_linter_err!(input, QError::InvalidConstant, 3, 31);
        }

        #[test]
        fn referencing_non_existing_member() {
            let input = "
            TYPE Card
                Suit AS STRING * 9
                Value AS INTEGER
            END TYPE

            DIM c AS Card
            PRINT c.Suite";
            // TODO QBasic reports the error at the dot
            assert_linter_err!(input, QError::ElementNotDefined, 8, 19);
        }

        #[test]
        fn user_defined_variable_name_cannot_include_period() {
            let input = "
            TYPE Card
                Suit AS STRING * 9
                Value AS INTEGER
            END TYPE

            DIM A.B AS Card
            ";
            assert_linter_err!(input, QError::IdentifierCannotIncludePeriod, 7, 17);
        }

        #[test]
        fn cannot_define_variable_with_dot_if_clashes_with_user_defined_type() {
            let input = "
            TYPE Card
                Suit AS STRING * 9
                Value AS INTEGER
            END TYPE

            DIM C AS Card
            DIM C.Oops AS STRING
            ";
            // QBasic actually throws "Expected: , or end-of-statement" at the period position
            assert_linter_err!(
                input,
                QError::syntax_error("Expected: , or end-of-statement"),
                8,
                17
            );
        }

        #[test]
        fn cannot_define_variable_with_dot_if_clashes_with_user_defined_type_reverse() {
            let input = "
            TYPE Card
                Suit AS STRING * 9
                Value AS INTEGER
            END TYPE

            DIM C.Oops AS STRING
            DIM C AS Card
            ";
            // QBasic actually throws "Expected: , or end-of-statement" at the period position
            assert_linter_err!(
                input,
                QError::syntax_error("Expected: , or end-of-statement"),
                8,
                17
            );
        }

        #[test]
        fn cannot_use_in_binary_expression() {
            let ops = [
                "=", "<>", ">=", ">", "<", "<=", "+", "-", "*", "/", "AND", "OR",
            ];
            for op in &ops {
                let input = format!(
                    "
                    TYPE Card
                        Value AS INTEGER
                    END TYPE

                    DIM a AS CARD
                    DIM b AS CARD

                    IF a {} b THEN
                    END IF",
                    op
                );
                // QBasic uses the right side expr for the location
                assert_linter_err!(input, QError::TypeMismatch, 9, 26 + (op.len() as u32) + 1);
            }
        }

        #[test]
        fn cannot_use_in_unary_expression() {
            let ops = ["-", "NOT "];
            for op in &ops {
                let input = format!(
                    "
                    TYPE Card
                        Value AS INTEGER
                    END TYPE

                    DIM a AS CARD
                    DIM b AS CARD

                    b = {}A",
                    op
                );
                assert_linter_err!(input, QError::TypeMismatch, 9, 25);
            }
        }
    }
}
// TODO test file handle expression cannot be used anywhere except for `OPEN`, `CLOSE`, `LINE INPUT`, `INPUT`
