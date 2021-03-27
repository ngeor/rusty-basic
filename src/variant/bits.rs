use crate::common::QError;
use std::cmp::{max, min};
use std::fmt::{Formatter, Write};

const INT_BITS: usize = 16;
const LONG_BITS: usize = 32;
const FLOAT_BITS: usize = 32;
const DOUBLE_BITS: usize = 64;

#[derive(Clone, PartialEq)]
pub struct BitVec {
    // msb -> lsb
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

    pub fn is_integer_size(&self) -> bool {
        self.len() == INT_BITS
    }

    pub fn is_long_size(&self) -> bool {
        self.len() == LONG_BITS
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
        Self { v: result.into() }
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

impl From<f64> for BitVec {
    fn from(value: f64) -> Self {
        let mut result: [bool; DOUBLE_BITS] = [false; DOUBLE_BITS];
        //  1 bit for sign,
        //
        // 11 bit for exponent,
        //
        // 52 bit for significant.
        //
        // 1023 for bias
        //
        // 1.significant * 2 ^ exponent

        let sign_bit: bool = value < 0.0_f64;
        let absolute_value = value.abs();

        let mut int_value: i64 = absolute_value.trunc() as i64;
        let mut int_bits: Vec<bool> = vec![];
        while int_value > 0 {
            let remainder = int_value % 2;
            int_value = int_value / 2;
            int_bits.push(remainder == 1);
        }
        println!("{:?}", int_bits);

        let mut fraction_value = absolute_value.fract();
        let mut fraction_bits: Vec<bool> = vec![];
        while fraction_bits.len() <= DOUBLE_BITS && (fraction_value - 1.0).abs() > 0.0 {
            fraction_value = fraction_value * 2.0;
            fraction_bits.push(fraction_value > 0.0);
        }
        println!("{:?}", fraction_bits);

        let mut exponent = if int_bits.is_empty() {
            0
        } else {
            (int_bits.len() - 1) as i32
        };
        println!("exponent {}", exponent);
        result[result.len() - 1] = sign_bit;
        for i in 0..11 {
            let remainder = exponent % 2;
            exponent = exponent / 2;
            result[result.len() - 2 - i] = remainder == 1;
        }

        int_bits.pop(); // the 1. part is always implied
        for i in 0..52 {
            if !int_bits.is_empty() {
                result[result.len() - 13 - i] = int_bits.pop().unwrap_or_default();
            } else if !fraction_bits.is_empty() {
                result[result.len() - 13 - i] = fraction_bits.pop().unwrap_or_default();
            }
        }

        Self { v: result.into() }
    }
}

impl From<BitVec> for f64 {
    fn from(bits: BitVec) -> Self {
        unimplemented!()
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
    let a_bits: BitVec = a.into();
    let b_bits: BitVec = b.into();
    let result = a_bits & b_bits;
    result.into()
}

pub fn qb_or(a: i32, b: i32) -> i32 {
    let a_bits: BitVec = a.into();
    let b_bits: BitVec = b.into();
    let result = a_bits | b_bits;
    result.into()
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

fn i32_to_bytes(i: i32) -> [u8; 2] {
    // BitVec is msb -> lsb
    let BitVec { v } = BitVec::from(i);
    debug_assert_eq!(INT_BITS, v.len());
    let high_byte = extract_byte(&v[0..8]);
    let low_byte = extract_byte(&v[8..16]);
    // result is lsb -> msb
    [low_byte, high_byte]
}

fn bytes_to_i32(b: [u8; 2]) -> i32 {
    todo!()
}

fn f64_to_bytes(f: f64) -> [u8; 8] {
    // LSB -> MSB
    todo!()
}

fn bytes_to_f64(b: [u8; 8]) -> f64 {
    todo!()
}

fn extract_byte(bits: &[bool]) -> u8 {
    // bits is encoded msb -> lsb
    debug_assert_eq!(8, bits.len());
    let mut result: u8 = 0;
    let mut mask: u8 = 0x80; // msb set
    for i in 0..bits.len() {
        result = result & mask;
        if i < bits.len() - 1 {
            mask = mask >> 1;
        }
    }
    result
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

    #[test]
    fn convert_f64() {
        // assert_eq!(
        //     format!("{}", BitVec::from(0.0_f64)),
        //     "00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000",
        //     "zero"
        // );
        // assert_eq!(
        //     format!("{}", BitVec::from(2.0_f64)),
        //     "00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000010",
        //     "2.0"
        // );
        // assert_eq!(
        //     format!("{}", BitVec::from(-2.0_f64)),
        //     "00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000011",
        //     "-2.0"
        // );
        //
        // assert_eq!(
        //     BitVec::from(-10.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 24 c0",
        //     "-10 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(-9.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 22 c0",
        //     "-9 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(-8.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 20 c0",
        //     "-8 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(-7.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 1c c0",
        //     "-7 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(-6.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 18 c0",
        //     "-6 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(-5.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 14 c0",
        //     "-5 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(-4.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 10 c0",
        //     "-4 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(-3.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 08 c0",
        //     "-3 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(-2.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 00 c0",
        //     "-2 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(-1.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 f0 3f",
        //     "-1 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(0.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 00 00",
        //     "zero as hex"
        // );
        // assert_eq!(
        //     BitVec::from(1.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 f0 3f",
        //     "1 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(2.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 00 40",
        //     "2 as hex"
        // );

        // 40 : 0100 0000
        // 08 : 0000 1000
        // 10 : 0001 0000

        // 3 should be 08 40:
        // 0 | 100_0000  0000  | 1000  (1.1 * 2 ^ 1)
        // assert_eq!(
        //     BitVec::from(3.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 08 40",
        //     "3 as hex"
        // );

        // 4 should be 10 40:
        // 0 | 100_0000 0001 | 0000 (1.0 * 2 ^ 1025) ? 1025 = 2 + 1023 (bias)
        // assert_eq!(
        //     BitVec::from(4.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 10 40",
        //     "4 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(5.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 14 40",
        //     "5 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(6.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 18 40",
        //     "6 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(7.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 1c 40",
        //     "7 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(8.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 20 40",
        //     "8 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(9.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 22 40",
        //     "9 as hex"
        // );
        // assert_eq!(
        //     BitVec::from(10.0_f64).to_hex_string(),
        //     "00 00 00 00 00 00 24 40",
        //     "10 as hex"
        // );
    }
}
