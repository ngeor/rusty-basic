use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::variant::QBNumberCast;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    if interpreter.context().variables().len() == 0 {
        interpreter.set_def_seg(None);
        Ok(())
    } else {
        let address: i64 = interpreter.context()[0].try_cast()?;
        if (0..=65535).contains(&address) {
            interpreter.set_def_seg(Some(address as usize));
            Ok(())
        } else {
            Err(QError::IllegalFunctionCall)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::{assert_interpreter_err, assert_prints};

    #[test]
    fn address_cannot_be_negative() {
        let input = "DEF SEG = -1";
        assert_interpreter_err!(input, QError::IllegalFunctionCall, 1, 1);
    }

    #[test]
    fn address_cannot_exceed_65535() {
        let input = "DEF SEG = 65536";
        assert_interpreter_err!(input, QError::IllegalFunctionCall, 1, 1);
    }

    #[test]
    fn happy_flow() {
        let input = r#"
        DIM A AS INTEGER
        P = VARPTR(A)
        DEF SEG = VARSEG(A)
        POKE P, 2     ' sets the low byte of A to 2
        POKE P + 1, 1 ' sets the high byte of A to 1
        PRINT A       ' result is 2 + 1 * 256 = 258
        "#;
        assert_prints!(input, "258");
    }

    #[test]
    fn caps_lock() {
        let input = r#"
        DEF SEG = 0                     ' Turn off CapLock, NumLock and ScrollLock
        KeyFlags = PEEK(1047)
        POKE 1047, &H0
        DEF SEG
        "#;
        assert_prints!(input, "");
    }

    #[test]
    fn data_poke() {
        let input = r#"
        DEFINT A-Z
        DATA 1, 2, 3, 4
        DIM A(1 TO 2)
        DEF SEG = VARSEG(A(1))
        FOR I = 1 TO 4
            READ X
            POKE VARPTR(A(1)) + I - 1, X
        NEXT
        DEF SEG
        PRINT A(1)
        PRINT A(2)
        "#;
        assert_prints!(input, "513", "1027");
    }
}
