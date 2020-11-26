use super::*;

pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), QErrorNode> {
    panic!("Should have been handled at the IG level")
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::QError;

    #[test]
    fn test_sub_call_system_no_args_allowed() {
        assert_linter_err!("SYSTEM 42", QError::ArgumentCountMismatch, 1, 1);
    }
}
