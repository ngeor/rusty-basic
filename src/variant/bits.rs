use crate::common::QError;
use crate::variant::Variant;
use std::fmt::{Formatter, Write};

const INT_BITS: usize = 16;
const LONG_BITS: usize = 32;
// const FLOAT_BITS: usize = 32;
const DOUBLE_BITS: usize = 64;

const DOUBLE_EXPONENT_BITS: usize = 11;
const DOUBLE_SIGNIFICANT_BITS: usize = 52;
const DOUBLE_BIAS: i32 = 1023;

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

    pub fn convert_to_integer_variant(mut self) -> Result<Variant, QError> {
        match Self::find_first_non_zero_bit(&self.v) {
            Some(idx) => {
                let bit_count = self.len() - idx;
                if bit_count == 0 {
                    // optimization
                    Ok(Variant::VInteger(0))
                } else if bit_count <= INT_BITS {
                    if bit_count < INT_BITS {
                        // inject one bit for the sign bit
                        self.v.insert(0, false);
                    }
                    let i: i32 = bits_to_i32(&self.v[idx..]);
                    Ok(Variant::VInteger(i))
                } else if bit_count <= LONG_BITS {
                    // it fits in a long
                    if bit_count < LONG_BITS {
                        // inject one bit for the sign bit
                        self.v.insert(0, false);
                    }
                    let l: i64 = bits_to_i64(&self.v[idx..]);
                    Ok(Variant::VLong(l))
                } else {
                    Err(QError::Overflow)
                }
            }
            None => Ok(Variant::VInteger(0)),
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

macro_rules! bits_to_integer_type {
    ($fn_name: tt, $integer_type: tt, $max_bits: expr) => {
        fn $fn_name(bits: &[bool]) -> $integer_type {
            debug_assert!(bits.len() <= $max_bits);
            debug_assert!(!bits.is_empty());
            let mut x: $integer_type = 0;
            let sign = bits[0];
            let mut idx = 1;
            while idx < bits.len() {
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
    };
}

bits_to_integer_type!(bits_to_i32, i32, INT_BITS);
bits_to_integer_type!(bits_to_i64, i64, LONG_BITS);

impl From<BitVec> for i32 {
    fn from(bits: BitVec) -> i32 {
        if bits.len() != INT_BITS {
            panic!("should be {} bits, was {}", INT_BITS, bits.len());
        }
        bits_to_i32(bits.v.as_slice())
    }
}

impl From<BitVec> for i64 {
    fn from(bits: BitVec) -> i64 {
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

pub fn i32_to_bytes(i: i32) -> [u8; 2] {
    // BitVec is msb -> lsb
    let BitVec { v } = BitVec::from(i);
    debug_assert_eq!(INT_BITS, v.len());
    let high_byte = msb_bits_to_byte(&v[0..8]);
    let low_byte = msb_bits_to_byte(&v[8..16]);
    // result is lsb -> msb
    [low_byte, high_byte]
}

pub fn bytes_to_i32(b: [u8; 2]) -> i32 {
    // given array is [low_byte, high_byte]
    // bits vector is msb -> lsb
    let bits: Vec<bool> = lsb_bytes_to_msb_bits(&b);
    debug_assert_eq!(INT_BITS, bits.len());
    let bit_vec = BitVec { v: bits };
    bit_vec.into()
}

pub fn f64_to_bytes(f: f64) -> [u8; 8] {
    // bits is msb -> lsb
    let bits = f64_to_bits(f);
    debug_assert_eq!(DOUBLE_BITS, bits.len());
    // result is lsb -> msb
    [
        msb_bits_to_byte(&bits[56..64]),
        msb_bits_to_byte(&bits[48..56]),
        msb_bits_to_byte(&bits[40..48]),
        msb_bits_to_byte(&bits[32..40]),
        msb_bits_to_byte(&bits[24..32]),
        msb_bits_to_byte(&bits[16..24]),
        msb_bits_to_byte(&bits[8..16]),
        msb_bits_to_byte(&bits[0..8]),
    ]
}

fn f64_to_bits(value: f64) -> Vec<bool> {
    // msb -> lsb
    let mut bits: Vec<bool> = vec![];
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
    bits.push(sign_bit);

    let absolute_value = value.abs();

    // create int_bits msb -> lsb
    // e.g. int_bits for 4 will be [1, 0, 0]
    let mut int_value: i64 = absolute_value.trunc() as i64;
    let mut int_bits: Vec<bool> = vec![];
    while int_value > 0 {
        let remainder = int_value % 2;
        int_bits.insert(0, remainder == 1);
        int_value = int_value / 2;
    }

    let mut fraction_value = absolute_value.fract();
    let mut fraction_bits: Vec<bool> = vec![];
    while fraction_bits.len() <= DOUBLE_BITS && (fraction_value - 1.0).abs() > 0.0 {
        fraction_value = fraction_value * 2.0;
        fraction_bits.push(fraction_value > 0.0);
    }

    let mut exponent = if int_bits.is_empty() {
        0
    } else {
        (int_bits.len() as i32) - 1 + DOUBLE_BIAS
    };
    for _i in 0..DOUBLE_EXPONENT_BITS {
        let remainder = exponent % 2;
        exponent = exponent / 2;
        bits.insert(1, remainder == 1);
    }

    if !int_bits.is_empty() {
        int_bits.remove(0); // the 1. part is always implied
    }
    for _i in 0..DOUBLE_SIGNIFICANT_BITS {
        let bit = if !int_bits.is_empty() {
            int_bits.remove(0)
        } else if !fraction_bits.is_empty() {
            fraction_bits.pop().unwrap_or_default()
        } else {
            false
        };
        bits.push(bit);
    }

    bits
}

pub fn bytes_to_f64(bytes: &[u8]) -> f64 {
    // bytes is lsb -> msb
    // bits is msb -> lsb
    let bits: Vec<bool> = lsb_bytes_to_msb_bits(bytes);
    debug_assert_eq!(bits.len(), DOUBLE_BITS);
    let sign = bits[0];
    let exponent_bits = &bits[1..12];
    let mut exponent: i32 = 0;
    for i in 0..exponent_bits.len() {
        if exponent_bits[i] {
            exponent += 2_i32.pow((exponent_bits.len() - i) as u32 - 1)
        }
    }

    if exponent == 0 {
        return 0.0;
    }

    let significant_bits = &bits[12..DOUBLE_BITS];
    // 1.significant * 2 ^ exponent - bias
    let mut result: f64 = 1.0;
    for i in 0..significant_bits.len() {
        if significant_bits[i] {
            result += 1.0 / 2.0_f64.powi(i as i32 + 1);
        }
    }
    result *= 2.0_f64.powi(exponent - DOUBLE_BIAS);
    if sign {
        -result
    } else {
        result
    }
}

/// Converts the given bit array into a byte.
/// The input array must be 8 bits long.
/// The input array is ordered from msb to lsb.
fn msb_bits_to_byte(bits: &[bool]) -> u8 {
    // bits is encoded msb -> lsb
    debug_assert_eq!(8, bits.len());
    let mut result: u8 = 0;
    let mut mask: u8 = 0x80; // msb set
    for i in 0..bits.len() {
        if bits[i] {
            result |= mask;
        }
        if i < bits.len() - 1 {
            mask >>= 1;
        }
    }
    result
}

/// Converts the given byte array into a bit array.
/// Every byte will be stored as 8 bits.
/// The byte array is ordered from low byte to high byte.
/// The resulting bit array is ordered from msb to lsb,
/// so the order of the bytes will be reversed.
fn lsb_bytes_to_msb_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits: Vec<bool> = vec![];
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        let mut mask: u8 = 0x80;
        let byte = bytes[i];
        while mask >= 0x01 {
            bits.push(byte & mask == mask);
            mask >>= 1;
        }
    }
    bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variant::{MAX_INTEGER, MIN_INTEGER};

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
    fn test_qb_and() {
        assert_eq!(4, qb_and(5, -2));
        assert_eq!(2, qb_and(-5, 2));
    }

    #[test]
    fn test_qb_or() {
        assert_eq!(3, qb_or(1, 2));
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
    fn test_i32_to_bytes() {
        assert_eq!(i32_to_bytes(0), [0, 0]);
        assert_eq!(i32_to_bytes(1), [1, 0]);
        assert_eq!(i32_to_bytes(2), [2, 0]);
        assert_eq!(i32_to_bytes(5), [5, 0]);
        assert_eq!(i32_to_bytes(255), [255, 0]);
        assert_eq!(i32_to_bytes(256), [0, 1]);
        assert_eq!(i32_to_bytes(-1), [255, 255]);
    }

    #[test]
    fn test_bytes_to_i32() {
        for i in MIN_INTEGER..(MAX_INTEGER + 1) {
            let bytes = i32_to_bytes(i);
            let converted_int = bytes_to_i32(bytes);
            assert_eq!(i, converted_int);
        }
    }

    #[test]
    fn test_f64_to_bytes() {
        assert_eq!(f64_to_bytes(-10.0), [0, 0, 0, 0, 0, 0, 0x24, 0xc0], "-10");
        assert_eq!(f64_to_bytes(-9.0), [0, 0, 0, 0, 0, 0, 0x22, 0xc0], "-9");
        assert_eq!(f64_to_bytes(-8.0), [0, 0, 0, 0, 0, 0, 0x20, 0xc0], "-8");
        assert_eq!(f64_to_bytes(-7.0), [0, 0, 0, 0, 0, 0, 0x1c, 0xc0], "-7");
        assert_eq!(f64_to_bytes(-6.0), [0, 0, 0, 0, 0, 0, 0x18, 0xc0], "-6");
        assert_eq!(f64_to_bytes(-5.0), [0, 0, 0, 0, 0, 0, 0x14, 0xc0], "-5");
        assert_eq!(f64_to_bytes(-4.0), [0, 0, 0, 0, 0, 0, 0x10, 0xc0], "-4");
        assert_eq!(f64_to_bytes(-3.0), [0, 0, 0, 0, 0, 0, 0x08, 0xc0], "-3");
        assert_eq!(f64_to_bytes(-2.0), [0, 0, 0, 0, 0, 0, 0x00, 0xc0], "-2");
        assert_eq!(f64_to_bytes(-1.0), [0, 0, 0, 0, 0, 0, 0xf0, 0xbf], "-1");
        assert_eq!(f64_to_bytes(0.0), [0, 0, 0, 0, 0, 0, 0x00, 0x00], "0");
        assert_eq!(f64_to_bytes(1.0), [0, 0, 0, 0, 0, 0, 0xf0, 0x3f], "1");
        assert_eq!(f64_to_bytes(2.0), [0, 0, 0, 0, 0, 0, 0x00, 0x40], "2");
        assert_eq!(f64_to_bytes(3.0), [0, 0, 0, 0, 0, 0, 0x08, 0x40], "3");
        assert_eq!(f64_to_bytes(4.0), [0, 0, 0, 0, 0, 0, 0x10, 0x40], "4");
        assert_eq!(f64_to_bytes(5.0), [0, 0, 0, 0, 0, 0, 0x14, 0x40], "5");
        assert_eq!(f64_to_bytes(6.0), [0, 0, 0, 0, 0, 0, 0x18, 0x40], "6");
        assert_eq!(f64_to_bytes(7.0), [0, 0, 0, 0, 0, 0, 0x1c, 0x40], "7");
        assert_eq!(f64_to_bytes(8.0), [0, 0, 0, 0, 0, 0, 0x20, 0x40], "8");
        assert_eq!(f64_to_bytes(9.0), [0, 0, 0, 0, 0, 0, 0x22, 0x40], "9");
        assert_eq!(f64_to_bytes(10.0), [0, 0, 0, 0, 0, 0, 0x24, 0x40], "10");
    }

    #[test]
    fn test_bytes_to_f64() {
        // let's try some integers
        for i in -MIN_INTEGER..MAX_INTEGER + 1 {
            let source: f64 = i as f64;
            let bytes = f64_to_bytes(source);
            let converted = bytes_to_f64(&bytes);
            assert_eq!(source, converted, "{}", source);
        }
    }
}
