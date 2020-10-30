use crate::common::QError;
use crate::variant::Variant;

#[derive(Clone, Debug)]
pub struct VArray {
    pub dimensions: Vec<(i32, i32)>,
    pub elements: Vec<Variant>,
}

impl VArray {
    pub fn get_element(&self, indices: Vec<i32>) -> Result<&Variant, QError> {
        let index = self.abs_index(indices)?;
        match self.elements.get(index) {
            Some(v) => Ok(v),
            _ => Err(QError::SubscriptOutOfRange),
        }
    }

    pub fn get_element_mut(&mut self, indices: Vec<i32>) -> Result<&mut Variant, QError> {
        let index = self.abs_index(indices)?;
        match self.elements.get_mut(index) {
            Some(v) => Ok(v),
            _ => Err(QError::SubscriptOutOfRange),
        }
    }

    fn abs_index(&self, indices: Vec<i32>) -> Result<usize, QError> {
        if indices.len() != self.dimensions.len() {
            return Err(QError::InternalError("Array indices mismatch".to_string()));
        }
        let mut index: i32 = 0;
        let mut i: i32 = indices.len() as i32 - 1;
        let mut multiplier: i32 = 1;
        while i >= 0 {
            let arg = indices[i as usize];
            let (lbound, ubound) = self.dimensions[i as usize];
            if arg < lbound || arg > ubound {
                return Err(QError::SubscriptOutOfRange);
            }

            index += (arg - lbound) * multiplier;
            multiplier = multiplier * (ubound - lbound + 1);
            i -= 1;
        }
        Ok(index as usize)
    }
}
