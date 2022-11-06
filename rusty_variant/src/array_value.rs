use crate::Variant;

#[derive(Clone, Debug)]
pub struct VArray {
    dimensions: Vec<(i32, i32)>,
    elements: Vec<Variant>,
}

pub struct SubscriptOutOfRangeError;

impl VArray {
    pub fn new(dimensions: Vec<(i32, i32)>, default_variant: Variant) -> Self {
        let len = dimensions_to_array_length(&dimensions);
        debug_assert!(len > 0);
        let elements: Vec<Variant> = (0..len).map(|_| default_variant.clone()).collect();
        Self {
            dimensions,
            elements,
        }
    }

    pub fn get_element(&self, indices: &[i32]) -> Result<&Variant, SubscriptOutOfRangeError> {
        let index = self.abs_index(indices)?;
        match self.elements.get(index) {
            Some(v) => Ok(v),
            _ => Err(SubscriptOutOfRangeError),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Variant> {
        self.elements.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Variant> {
        self.elements.get_mut(index)
    }

    pub fn get_element_mut(
        &mut self,
        indices: &[i32],
    ) -> Result<&mut Variant, SubscriptOutOfRangeError> {
        let index = self.abs_index(indices)?;
        match self.elements.get_mut(index) {
            Some(v) => Ok(v),
            _ => Err(SubscriptOutOfRangeError),
        }
    }

    /// Maps the indices of a multi-dimensional array element into a flat index.
    pub fn abs_index(&self, indices: &[i32]) -> Result<usize, SubscriptOutOfRangeError> {
        debug_assert_eq!(indices.len(), self.dimensions.len());
        let mut index: i32 = 0;
        let mut i: i32 = indices.len() as i32 - 1;
        let mut multiplier: i32 = 1;
        while i >= 0 {
            let arg = indices[i as usize];
            let (lbound, ubound) = self.dimensions[i as usize];
            if arg < lbound || arg > ubound {
                return Err(SubscriptOutOfRangeError);
            }

            index += (arg - lbound) * multiplier;
            multiplier *= ubound - lbound + 1;
            i -= 1;
        }
        Ok(index as usize)
    }

    /// Returns the lower and upper bounds of the given dimension.
    pub fn get_dimension_bounds(&self, dimension_index: usize) -> Option<&(i32, i32)> {
        self.dimensions.get(dimension_index)
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn first(&self) -> &Variant {
        self.elements
            .first()
            .expect("Empty arrays are not supported")
    }
}

/// Calculates the number of elements in a multi-dimensional array.
fn dimensions_to_array_length(dimensions: &[(i32, i32)]) -> usize {
    let mut len: usize = 1;
    for (lbound, ubound) in dimensions {
        len *= (*ubound - *lbound + 1) as usize;
    }
    len
}
