use crate::RuntimeError;
use rusty_variant::Variant;

#[derive(Default)]
pub struct DataSegment {
    values: Vec<Variant>,
    index: usize,
}

impl DataSegment {
    pub fn push(&mut self, value: Variant) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Result<Variant, RuntimeError> {
        match self.values.get(self.index) {
            Some(v) => {
                self.index += 1;
                Ok(v.clone())
            }
            _ => Err(RuntimeError::OutOfData),
        }
    }
}
