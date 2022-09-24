use crate::common::QError;
use crate::variant::Variant;

pub struct DataSegment {
    v: Vec<Variant>,
    idx: usize,
}

impl DataSegment {
    pub fn new() -> Self {
        Self { v: vec![], idx: 0 }
    }

    pub fn push(&mut self, v: Variant) {
        self.v.push(v);
    }

    pub fn pop(&mut self) -> Result<Variant, QError> {
        match self.v.get(self.idx) {
            Some(v) => {
                self.idx += 1;
                Ok(v.clone())
            }
            _ => Err(QError::OutOfData),
        }
    }
}
