use crate::common::QError;
use crate::variant::{AsciiSize, Variant};

#[derive(Clone, Debug)]
pub struct VArray {
    dimensions: Vec<(i32, i32)>,
    elements: Vec<Variant>,
}

impl VArray {
    pub fn new(dimensions: Vec<(i32, i32)>, default_variant: Variant) -> Self {
        let len = dimensions_to_array_length(&dimensions);
        let elements: Vec<Variant> = (0..len).map(|_| default_variant.clone()).collect();
        Self {
            dimensions,
            elements,
        }
    }

    pub fn get_element(&self, indices: &[i32]) -> Result<&Variant, QError> {
        let index = self.abs_index(indices)?;
        match self.elements.get(index) {
            Some(v) => Ok(v),
            _ => Err(QError::SubscriptOutOfRange),
        }
    }

    pub fn get_element_mut(&mut self, indices: &[i32]) -> Result<&mut Variant, QError> {
        let index = self.abs_index(indices)?;
        match self.elements.get_mut(index) {
            Some(v) => Ok(v),
            _ => Err(QError::SubscriptOutOfRange),
        }
    }

    /// Maps the indices of a multi-dimensional array element into a flat index.
    fn abs_index(&self, indices: &[i32]) -> Result<usize, QError> {
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

    /// Returns the lower and upper bounds of the given dimension.
    pub fn get_dimension_bounds(&self, dimension_index: usize) -> Option<&(i32, i32)> {
        self.dimensions.get(dimension_index)
    }

    pub fn address_offset_of_element(&self, indices: &[i32]) -> Result<usize, QError> {
        let abs_index = self.abs_index(indices)?;
        Ok(self.element_size_in_bytes() * abs_index)
    }

    fn element_size_in_bytes(&self) -> usize {
        self.elements
            .first()
            .map(Variant::ascii_size)
            .unwrap_or_default()
    }

    pub fn peek_array_element(&self, address: usize) -> Result<u8, QError> {
        let element_size = self.element_size_in_bytes();
        if element_size == 0 {
            Err(QError::SubscriptOutOfRange)
        } else {
            let element_index = address / element_size;
            let offset = address % element_size;
            let element = self
                .elements
                .get(element_index)
                .ok_or(QError::SubscriptOutOfRange)?;
            element.peek_non_array(offset)
        }
    }

    pub fn poke_array_element(&mut self, address: usize, value: u8) -> Result<(), QError> {
        let element_size = self.element_size_in_bytes();
        if element_size == 0 {
            Err(QError::SubscriptOutOfRange)
        } else {
            let element_index = address / element_size;
            let offset = address % element_size;
            let element = self
                .elements
                .get_mut(element_index)
                .ok_or(QError::SubscriptOutOfRange)?;
            element.poke_non_array(offset, value)
        }
    }
}

impl AsciiSize for VArray {
    fn ascii_size(&self) -> usize {
        let array_length = dimensions_to_array_length(&self.dimensions);
        self.element_size_in_bytes() * array_length
    }
}

/// Calculates the number of elements in a multi-dimensional array.
fn dimensions_to_array_length(dimensions: &[(i32, i32)]) -> usize {
    let mut len: usize = 1;
    for (lbound, ubound) in dimensions {
        len = len * ((*ubound - *lbound + 1) as usize);
    }
    len
}
