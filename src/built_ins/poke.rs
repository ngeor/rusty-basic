pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNodes;

    pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
        if args.len() != 2 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            for i in 0..args.len() {
                args.require_integer_argument(i)?;
            }
            Ok(())
        }
    }
}

pub mod interpreter {
    use crate::built_ins::peek::interpreter::INDICATOR_KEYS_ADDRESS;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::keyboard::set_indicator_keys;
    use crate::interpreter::utils::VariantCasts;
    use crate::variant::QBNumberCast;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let address: usize = interpreter.context()[0].to_non_negative_int()?;
        let value: i32 = interpreter.context()[1].try_cast()?;
        if value >= 0 && value < 256 {
            let b: u8 = value as u8;
            let seg = interpreter.get_def_seg_or_default();
            if seg == 0 {
                zero_seg(address, b)
            } else {
                interpreter.context_mut().poke(seg, address, b)
            }
        } else {
            Err(QError::IllegalFunctionCall)
        }
    }

    fn zero_seg(address: usize, value: u8) -> Result<(), QError> {
        if address == INDICATOR_KEYS_ADDRESS {
            unsafe { set_indicator_keys(value) }
        } else {
            unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::QError;

    #[test]
    fn must_have_arguments() {
        let input = "POKE";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }

    #[test]
    fn one_argument() {
        let input = "POKE 42";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }

    #[test]
    fn three_arguments() {
        let input = "POKE 1, 2, 3";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }

    #[test]
    fn string_first_argument() {
        let input = "POKE A$, 1";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn string_second_argument() {
        let input = "POKE 1, A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }
}
