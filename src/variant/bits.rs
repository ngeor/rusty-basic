use crate::common::QError;
use crate::variant::{INT_BITS, LONG_BITS};

#[derive(Clone, Debug, PartialEq)]
pub struct BitVec {
    v: Vec<bool>,
}

impl BitVec {
    pub fn new() -> Self {
        Self { v: vec![] }
    }

    pub fn len(&self) -> usize {
        self.v.len()
    }

    pub fn push_hex(&mut self, u: u8) {
        self.v.push(u & 8 == 8);
        self.v.push(u & 4 == 4);
        self.v.push(u & 2 == 2);
        self.v.push(u & 1 == 1);
    }

    pub fn push_oct(&mut self, u: u8) {
        self.v.push(u & 4 == 4);
        self.v.push(u & 2 == 2);
        self.v.push(u & 1 == 1);
    }

    pub fn fit(&mut self) -> Result<(), QError> {
        // find first non-zero bit
        let mut first_non_zero_bit: usize = 0;
        while first_non_zero_bit < self.v.len() && !self.v[first_non_zero_bit] {
            first_non_zero_bit += 1;
        }
        if self.v.len() - first_non_zero_bit <= INT_BITS {
            self.fit_to(INT_BITS);
            Ok(())
        } else if self.v.len() - first_non_zero_bit <= LONG_BITS {
            self.fit_to(LONG_BITS);
            Ok(())
        } else {
            Err(QError::Overflow)
        }
    }

    fn fit_to(&mut self, bits: usize) {
        if self.v.len() > bits {
            while self.v.len() > bits {
                self.v.remove(0);
            }
        } else if self.v.len() < bits {
            while self.v.len() < bits {
                self.v.insert(0, false);
            }
        }
    }
}

impl From<Vec<bool>> for BitVec {
    fn from(v: Vec<bool>) -> Self {
        Self { v }
    }
}

impl From<[bool; INT_BITS]> for BitVec {
    fn from(bits: [bool; 16]) -> Self {
        let v : Vec<bool> = bits.into();
        v.into()
    }
}

impl From<i32> for BitVec {
    fn from(a: i32) -> Self {
        let mut result: [bool; INT_BITS] = [false; INT_BITS];
        let mut x: i32 = a;
        let mut idx = INT_BITS;
        if x > 0 {
            while x > 0 && idx > 0 {
                idx -= 1;
                result[idx] = (x & 1) == 1;
                x = x >> 1;
            }
        } else if x < 0 {
            x = -x - 1;
            result = [true; INT_BITS];
            while x > 0 && idx > 0 {
                idx -= 1;
                result[idx] = (x & 1) == 0;
                x = x >> 1;
            }
        }
        result.into()
    }
}

impl From<BitVec> for i32 {
    fn from(bits: BitVec) -> i32 {
        if bits.len() != INT_BITS {
            panic!("should be {} bits, was {}", INT_BITS, bits.len());
        }
        let mut x: i32 = 0;
        let sign = bits[0];
        let mut idx = 1;
        while idx < INT_BITS {
            x = x << 1;
            if bits[idx] != sign {
                x = x | 1;
            }
            idx += 1;
        }
        if sign {
            -x - 1
        } else {
            x
        }
    }
}

impl From<BitVec> for i64 {
    fn from(bits: BitVec) -> i64 {
        if bits.len() != LONG_BITS {
            panic!("should be {} bits, was {}", LONG_BITS, bits.len());
        }
        let mut x: i64 = 0;
        let sign = bits[0];
        let mut idx = 1;
        while idx < LONG_BITS {
            x = x << 1;
            if bits[idx] != sign {
                x = x | 1;
            }
            idx += 1;
        }
        if sign {
            -x - 1
        } else {
            x
        }
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
    type Output = BitVec;

    fn bitand(self, rhs: Self) -> Self::Output {
        if self.len() != rhs.len() {
            panic!("Incompatible BitVec");
        }

        let mut result = BitVec::new();
        for i in 0..self.len() {
            result.v.push(self[i] && rhs[i]);
        }
        result.into()
    }
}

impl std::ops::BitOr for BitVec {
    type Output = BitVec;

    fn bitor(self, rhs: Self) -> Self::Output {
        if self.len() != rhs.len() {
            panic!("Incompatible BitVec");
        }
        let mut result = BitVec::new();
        for i in 0..self.len() {
            result.v.push(self[i] || rhs[i]);
        }
        result.into()
    }
}

pub fn qb_and(a: i32, b: i32) -> i32 {
    let a_bits : BitVec = a.into();
    let b_bits: BitVec = b.into();
    let result  = a_bits & b_bits;
    result.into()
}

pub fn qb_or(a: i32, b: i32) -> i32 {
    let a_bits : BitVec = a.into();
    let b_bits: BitVec = b.into();
    let result  = a_bits | b_bits;
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_bits() {
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
        expected_bits[INT_BITS - 2] = true;
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
    fn test_negative_bits() {
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
    fn test_from_to_bits() {
        for i in -5..6 {
            let bit_vec: BitVec = i.into();
            let j: i32 = bit_vec.into();
            assert_eq!(i, j);
        }
    }

    #[test]
    fn test_and_bits() {
        assert_eq!(4, qb_and(5, -2));
        assert_eq!(2, qb_and(-5, 2));
    }
}
