pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::parser::base::and_pc::AndDemandTrait;
    use crate::parser::base::parsers::{FnMapTrait, KeepMiddleTrait, Parser};
    use crate::parser::specific::{keyword_p, OrSyntaxErrorTrait};
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        keyword_p(Keyword::Name)
            .and_demand(
                expression::back_guarded_expression_node_p()
                    .or_syntax_error("Expected: old file name"),
            )
            .and_demand(keyword_p(Keyword::As).or_syntax_error("Expected: AS"))
            .keep_middle()
            .and_demand(
                expression::guarded_expression_node_p().or_syntax_error("Expected: new file name"),
            )
            .fn_map(|(l, r)| Statement::BuiltInSubCall(BuiltInSub::Name, vec![l, r]))
    }
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() != 2 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            args.require_string_argument(0)?;
            args.require_string_argument(1)
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let old_file_name: &str = interpreter.context()[0].to_str_unchecked();
        let new_file_name: &str = interpreter.context()[1].to_str_unchecked();
        std::fs::rename(old_file_name, new_file_name).map_err(QError::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::*;
    use crate::interpreter::test_utils::*;

    #[test]
    fn test_can_rename_file() {
        // arrange
        std::fs::write("TEST4.OLD", "hi").unwrap_or(());
        let input = r#"
        NAME "TEST4.OLD" AS "TEST4.NEW"
        "#;
        // act
        interpret(input);
        // assert
        let contents = std::fs::read_to_string("TEST4.NEW").unwrap_or("".to_string());
        std::fs::remove_file("TEST4.OLD").unwrap_or(());
        std::fs::remove_file("TEST4.NEW").unwrap_or(());
        assert_eq!(contents, "hi");
    }

    #[test]
    fn test_can_rename_file_parenthesis() {
        // arrange
        std::fs::write("TEST5.OLD", "hi").unwrap_or(());
        let input = r#"
        NAME("TEST5.OLD")AS("TEST5.NEW")
        "#;
        // act
        interpret(input);
        // assert
        let contents = std::fs::read_to_string("TEST5.NEW").unwrap_or("".to_string());
        std::fs::remove_file("TEST5.OLD").unwrap_or(());
        std::fs::remove_file("TEST5.NEW").unwrap_or(());
        assert_eq!(contents, "hi");
    }

    #[test]
    fn test_can_rename_directory() {
        // arrange
        let old_dir_name = "TEST.DIR";
        let new_dir_name = "NEW.DIR";
        std::fs::remove_dir(old_dir_name).unwrap_or(());
        std::fs::remove_dir(new_dir_name).unwrap_or(());
        std::fs::create_dir(old_dir_name).expect("Should have created directory");

        // act
        interpret(format!(r#"NAME "{}" AS "{}""#, old_dir_name, new_dir_name));

        // assert
        std::fs::metadata(old_dir_name).expect_err("should fail");
        let attr = std::fs::metadata(new_dir_name).expect("should succeed");
        assert!(attr.is_dir());
        std::fs::remove_dir(old_dir_name).unwrap_or(());
        std::fs::remove_dir(new_dir_name).unwrap_or(());
    }

    #[test]
    fn test_name_linter_err() {
        assert_linter_err!(r#"NAME 1 AS "boo""#, QError::ArgumentTypeMismatch, 1, 6);
        assert_linter_err!(r#"NAME "boo" AS 2"#, QError::ArgumentTypeMismatch, 1, 15);
    }
}
