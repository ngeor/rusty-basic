use super::peek::INDICATOR_KEYS_ADDRESS;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::keyboard::set_indicator_keys;
use crate::interpreter::utils::VariantCasts;
use rusty_common::*;
use rusty_linter::QBNumberCast;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let address: usize = interpreter.context()[0].to_non_negative_int()?;
    let value: i32 = interpreter.context()[1].try_cast()?;
    if (0..256).contains(&value) {
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
