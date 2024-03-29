use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;
use crate::RuntimeError;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let old_file_name: &str = interpreter.context()[0].to_str_unchecked();
    let new_file_name: &str = interpreter.context()[1].to_str_unchecked();
    std::fs::rename(old_file_name, new_file_name).map_err(RuntimeError::from)
}

#[cfg(test)]
mod tests {
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
        let contents = std::fs::read_to_string("TEST4.NEW").unwrap_or_default();
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
        let contents = std::fs::read_to_string("TEST5.NEW").unwrap_or_default();
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
        interpret(&format!(r#"NAME "{}" AS "{}""#, old_dir_name, new_dir_name));

        // assert
        std::fs::metadata(old_dir_name).expect_err("should fail");
        let attr = std::fs::metadata(new_dir_name).expect("should succeed");
        assert!(attr.is_dir());
        std::fs::remove_dir(old_dir_name).unwrap_or(());
        std::fs::remove_dir(new_dir_name).unwrap_or(());
    }
}
