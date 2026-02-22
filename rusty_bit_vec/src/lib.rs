use std::fmt::Write;

pub const MIN_INTEGER: i32 = -32768;
pub const MAX_INTEGER: i32 = 32767;
pub const MIN_LONG: i64 = -2147483648;
pub const MAX_LONG: i64 = 2147483647;

pub const INT_BITS: usize = 16;
pub const LONG_BITS: usize = 32;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct BitVec {
    // msb -> lsb
    v: Vec<bool>,
}

pub struct OverflowError;

impl BitVec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.v.len()
    }

    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }

    pub fn push_hex(&mut self, u: u8) {
        // msb -> lsb
        self.v.push(u & 8 == 8);
        self.v.push(u & 4 == 4);
        self.v.push(u & 2 == 2);
        self.v.push(u & 1 == 1);
    }

    pub fn push_oct(&mut self, u: u8) {
        // msb -> lsb
        self.v.push(u & 4 == 4);
        self.v.push(u & 2 == 2);
        self.v.push(u & 1 == 1);
    }

    pub fn convert_to_int_or_long_expr(mut self) -> Result<BitVecIntOrLong, OverflowError> {
        match Self::find_first_non_zero_bit(&self.v) {
            Some(index) => {
                let bit_count = self.len() - index;
                if bit_count == 0 {
                    // optimization
                    Ok(BitVecIntOrLong::Int(0))
                } else if bit_count <= INT_BITS {
                    if bit_count < INT_BITS {
                        // inject one bit for the sign bit
                        self.v.insert(0, false);
                    }
                    let i: i32 = bits_to_i32(&self.v[index..]);
                    Ok(BitVecIntOrLong::Int(i))
                } else if bit_count <= LONG_BITS {
                    // it fits in a long
                    if bit_count < LONG_BITS {
                        // inject one bit for the sign bit
                        self.v.insert(0, false);
                    }
                    let l: i64 = bits_to_i64(&self.v[index..]);
                    Ok(BitVecIntOrLong::Long(l))
                } else {
                    Err(OverflowError)
                }
            }
            None => Ok(BitVecIntOrLong::Int(0)),
        }
    }

    fn find_first_non_zero_bit(bits: &[bool]) -> Option<usize> {
        let mut index: usize = 0;
        while index < bits.len() && !bits[index] {
            index += 1;
        }
        if index < bits.len() {
            Some(index)
        } else {
            None
        }
    }
}

// TODO rename
pub enum BitVecIntOrLong {
    Int(i32),
    Long(i64),
}

impl From<i32> for BitVec {
    fn from(a: i32) -> Self {
        let mut result: [bool; INT_BITS] = [false; INT_BITS];
        let mut x: i32 = a;
        let mut index = INT_BITS;
        if x > 0 {
            while x > 0 && index > 0 {
                index -= 1;
                result[index] = (x & 1) == 1;
                x >>= 1;
            }
        } else if x < 0 {
            x = -x - 1;
            result = [true; INT_BITS];
            while x > 0 && index > 0 {
                index -= 1;
                result[index] = (x & 1) == 0;
                x >>= 1;
            }
        }
        Self { v: result.into() }
    }
}

impl From<BitVec> for i32 {
    fn from(bits: BitVec) -> Self {
        if bits.len() != INT_BITS {
            panic!("should be {} bits, was {}", INT_BITS, bits.len());
        }
        bits_to_i32(bits.v.as_slice())
    }
}

impl From<BitVec> for i64 {
    fn from(bits: BitVec) -> Self {
        if bits.len() != LONG_BITS {
            panic!("should be {} bits, was {}", LONG_BITS, bits.len());
        }
        bits_to_i64(bits.v.as_slice())
    }
}

impl std::ops::Index<usize> for BitVec {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        self.v.get(index).unwrap()
    }
}

impl std::ops::IndexMut<usize> for BitVec {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.v.get_mut(index).unwrap()
    }
}

impl std::ops::BitAnd for BitVec {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        if self.len() != rhs.len() {
            panic!("Incompatible BitVec");
        }

        let mut result = Self::new();
        for i in 0..self.len() {
            result.v.push(self[i] && rhs[i]);
        }
        result
    }
}

impl std::ops::BitOr for BitVec {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        if self.len() != rhs.len() {
            panic!("Incompatible BitVec");
        }
        let mut result = Self::new();
        for i in 0..self.len() {
            result.v.push(self[i] || rhs[i]);
        }
        result
    }
}

impl std::fmt::Display for BitVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.len() {
            let bit = self[i];
            let ch = if bit { '1' } else { '0' };
            if i % 4 == 0 && i > 0 {
                f.write_char(' ')?;
            }
            f.write_char(ch)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for BitVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl From<BitVec> for Vec<bool> {
    fn from(bits: BitVec) -> Self {
        bits.v
    }
}

impl From<Vec<bool>> for BitVec {
    fn from(bits: Vec<bool>) -> Self {
        Self { v: bits }
    }
}

macro_rules! bits_to_integer_type {
    ($fn_name: tt, $integer_type: tt, $max_bits: expr) => {
        fn $fn_name(bits: &[bool]) -> $integer_type {
            debug_assert!(bits.len() <= $max_bits);
            debug_assert!(!bits.is_empty());
            let mut x: $integer_type = 0;
            let sign = bits[0];
            let mut index = 1;
            while index < bits.len() {
                x <<= 1;
                if bits[index] != sign {
                    x |= 1;
                }
                index += 1;
            }
            if sign { -x - 1 } else { x }
        }
    };
}

bits_to_integer_type!(bits_to_i32, i32, INT_BITS);
bits_to_integer_type!(bits_to_i64, i64, LONG_BITS);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_vec_from_positive_int() {
        let mut expected_bits: BitVec = 0.into();

        // 0 | 0 0 0
        assert_eq!(BitVec::from(0), expected_bits);

        // 0 | 0 0 1
        expected_bits[INT_BITS - 1] = true;
        assert_eq!(BitVec::from(1), expected_bits);

        // 0 | 0 1 0
        expected_bits[INT_BITS - 1] = false;
        expected_bits[INT_BITS - 2] = true;
        assert_eq!(BitVec::from(2), expected_bits);

        // 0 | 0 1 1
        expected_bits[INT_BITS - 1] = true;
        assert_eq!(BitVec::from(3), expected_bits);

        // 0 | 1 0 0
        expected_bits[INT_BITS - 1] = false;
        expected_bits[INT_BITS - 2] = false;
        expected_bits[INT_BITS - 3] = true;
        assert_eq!(BitVec::from(4), expected_bits);

        // 0 | 1 0 1
        expected_bits[INT_BITS - 1] = true;
        assert_eq!(BitVec::from(5), expected_bits);
    }

    #[test]
    fn test_bit_vec_from_negative_int() {
        let mut expected_bits: BitVec = 0.into();
        for i in 0..expected_bits.len() {
            expected_bits[i] = true;
        }

        // 1 | 1 1 1
        assert_eq!(BitVec::from(-1), expected_bits);

        // 1 | 1 1 0
        expected_bits[INT_BITS - 1] = false;
        assert_eq!(BitVec::from(-2), expected_bits);

        // 1 | 1 0 1
        expected_bits[INT_BITS - 1] = true;
        expected_bits[INT_BITS - 2] = false;
        assert_eq!(BitVec::from(-3), expected_bits);

        // 1 | 1 0 0
        expected_bits[INT_BITS - 1] = false;
        assert_eq!(BitVec::from(-4), expected_bits);

        // 1 | 0 1 1
        expected_bits[INT_BITS - 1] = true;
        expected_bits[INT_BITS - 2] = true;
        expected_bits[INT_BITS - 3] = false;
        assert_eq!(BitVec::from(-5), expected_bits);
    }

    #[test]
    fn test_int_from_bit_vec() {
        for i in -5..6 {
            let bit_vec: BitVec = i.into();
            let j: i32 = bit_vec.into();
            assert_eq!(i, j);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", BitVec::from(0)), "0000 0000 0000 0000");
        assert_eq!(format!("{}", BitVec::from(1)), "0000 0000 0000 0001");
        assert_eq!(format!("{}", BitVec::from(2)), "0000 0000 0000 0010");
        assert_eq!(format!("{}", BitVec::from(3)), "0000 0000 0000 0011");
        assert_eq!(format!("{}", BitVec::from(4)), "0000 0000 0000 0100");
        assert_eq!(format!("{}", BitVec::from(5)), "0000 0000 0000 0101");
        assert_eq!(format!("{}", BitVec::from(32767)), "0111 1111 1111 1111");
        assert_eq!(format!("{}", BitVec::from(-32768)), "1000 0000 0000 0000");
        assert_eq!(format!("{}", BitVec::from(-1)), "1111 1111 1111 1111");
    }
}
