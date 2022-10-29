use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::keyboard::get_indicator_keys;
use crate::interpreter::utils::VariantCasts;
use crate::parser::BuiltInFunction;

pub const INDICATOR_KEYS_ADDRESS: usize = 1047;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let address: usize = interpreter.context()[0].to_non_negative_int()?;
    let seg = interpreter.get_def_seg_or_default();
    let peek_value: u8 = if seg == 0 {
        // use seg, special case if 0
        zero_seg(address)?
    } else {
        interpreter.context().peek(seg, address)?
    };
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Peek, peek_value as i32);
    Ok(())
}

fn zero_seg(address: usize) -> Result<u8, QError> {
    if address == INDICATOR_KEYS_ADDRESS {
        unsafe { get_indicator_keys() }
    } else {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn peek_without_def_seg() {
        let input = r#"
        DEFINT A-Z
        A = 256 + 42
        PRINT PEEK(VARPTR(A))
        PRINT PEEK(VARPTR(A) + 1)
        "#;
        assert_prints!(input, "42", "1");
    }

    #[test]
    fn peek_with_def_seg() {
        let input = r#"
        DEFINT A-Z
        DIM A(1 TO 2)
        A(1) = 42 + 256
        A(2) = 100
        DEF SEG = VARSEG(A(1))
        PRINT PEEK(VARPTR(A(1)))
        PRINT PEEK(VARPTR(A(1)) + 1)
        PRINT PEEK(VARPTR(A(2)))
        "#;
        assert_prints!(input, "42", "1", "100");
    }
}
