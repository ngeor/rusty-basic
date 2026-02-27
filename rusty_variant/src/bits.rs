use rusty_bit_vec::{BitVec, INT_BITS};

// const FLOAT_BITS: usize = 32;
const DOUBLE_BITS: usize = 64;

const DOUBLE_EXPONENT_BITS: usize = 11;
const DOUBLE_SIGNIFICANT_BITS: usize = 52;
const DOUBLE_BIAS: i32 = 1023;

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

pub fn i32_to_bytes(i: i32) -> [u8; 2] {
    // BitVec is msb -> lsb
    let bit_vec = BitVec::from(i);
    let v: Vec<bool> = bit_vec.into();
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
    let bit_vec = BitVec::from(bits);
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
    match f64_abs_normalize_value(value) {
        Some((absolute_value, initial_exponent)) => {
            f64_to_bits_for_normalized_value(value < 0.0, absolute_value, initial_exponent)
        }
        None => {
            // zero
            [false; DOUBLE_BITS].to_vec()
        }
    }
}

macro_rules! int_to_bits_vec {
    ($value: expr, $bits: expr, $bit_index: expr) => {{
        let mut temp = $value;
        while temp > 0 {
            let remainder = temp % 2;
            $bits.insert($bit_index, remainder == 1);
            temp /= 2;
        }
    }};
}

fn f64_to_bits_for_normalized_value(
    is_negative: bool,
    absolute_value: f64,
    initial_exponent: usize,
) -> Vec<bool> {
    // msb -> lsb
    //
    //  1 bit for sign,
    //
    // 11 bit for exponent,
    //
    // 52 bit for significant.
    //
    // 1023 for bias
    //
    // 1.significant * 2 ^ exponent
    let mut bits: Vec<bool> = vec![];
    bits.push(is_negative);

    // create int_bits msb -> lsb
    // e.g. int_bits for 4 will be [1, 0, 0], but we remove the 1., so it will be [0, 0]
    let int_bits = f64_int_bits(absolute_value);
    let fraction_bits = f64_fractional_bits(absolute_value);

    let exponent_with_bias = (int_bits.len() as i32) + DOUBLE_BIAS - (initial_exponent as i32);
    // insert the exponent bits
    int_to_bits_vec!(exponent_with_bias, bits, 1);
    // insert zeroes
    while bits.len() < 1 + DOUBLE_EXPONENT_BITS {
        bits.insert(1, false);
    }
    // make sure we didn't overflow the exponent bits
    debug_assert_eq!(
        1 + DOUBLE_EXPONENT_BITS,
        bits.len(),
        "Exponent bits overflow"
    );
    // insert the significant bits
    for bit in int_bits
        .into_iter()
        .chain(fraction_bits)
        .take(DOUBLE_SIGNIFICANT_BITS)
    {
        bits.push(bit);
    }
    debug_assert_eq!(DOUBLE_BITS, bits.len());
    bits
}

fn f64_int_bits(absolute_value: f64) -> Vec<bool> {
    let mut int_bits: Vec<bool> = vec![];
    int_to_bits_vec!(absolute_value.trunc() as i64, int_bits, 0);
    int_bits.remove(0); // it always starts with 1.
    int_bits
}

fn f64_fractional_bits(absolute_value: f64) -> Vec<bool> {
    let mut fraction_value = absolute_value.fract();
    let mut fraction_bits: Vec<bool> = vec![];
    while fraction_bits.len() <= DOUBLE_SIGNIFICANT_BITS {
        if fraction_value >= 0.5 {
            fraction_bits.push(true);
            fraction_value = fraction_value * 2.0 - 1.0;
        } else {
            fraction_bits.push(false);
            fraction_value *= 2.0;
        }
    }
    fraction_bits
}

fn f64_abs_normalize_value(value: f64) -> Option<(f64, usize)> {
    let mut absolute_value = value.abs();
    let mut exponent: usize = 0;
    while absolute_value < 1.0 && exponent < (DOUBLE_BIAS as usize) {
        absolute_value *= 2.0;
        exponent += 1;
    }

    if absolute_value < 1.0 {
        None
    } else {
        Some((absolute_value, exponent))
    }
}

#[cfg(test)]
fn fmt_double_bits(bits: &[bool]) -> String {
    let mut result = String::new();
    fmt_bits_into_string(&mut result, &bits[0..1]);
    result.push('|');
    fmt_bits_into_string(&mut result, &bits[1..DOUBLE_EXPONENT_BITS + 1]);
    result.push('|');
    fmt_bits_into_string(&mut result, &bits[1 + DOUBLE_EXPONENT_BITS..]);
    result
}

#[cfg(test)]
fn fmt_bits_into_string(s: &mut String, bits: &[bool]) {
    for bit in bits {
        s.push(if *bit { '1' } else { '0' });
    }
}

pub fn bytes_to_f64(bytes: &[u8]) -> f64 {
    // bytes is lsb -> msb
    // bits is msb -> lsb
    debug_assert_eq!(bytes.len(), DOUBLE_BITS / 8);
    let bits: Vec<bool> = lsb_bytes_to_msb_bits(bytes);
    debug_assert_eq!(bits.len(), DOUBLE_BITS);
    let sign = bits[0];

    let exponent_bits = &bits[1..DOUBLE_EXPONENT_BITS + 1];
    debug_assert_eq!(DOUBLE_EXPONENT_BITS, exponent_bits.len());
    let mut exponent_with_bias: i32 = 0;
    for exponent_bit in exponent_bits.iter() {
        exponent_with_bias *= 2;
        if *exponent_bit {
            exponent_with_bias += 1;
        }
    }

    // exponent_with_bias == 0 && F == 0 -> 0
    // exponent_with_bias == 0 && F!= 0 -> subnormals
    // exponent_with_bias == 0x7ff (all 1s) && F == 0 -> inf
    // exponent_with_bias == 0x7ff (all 1s) && F != 0 -> NaN

    let significant_bits = &bits[1 + DOUBLE_EXPONENT_BITS..];
    debug_assert_eq!(DOUBLE_SIGNIFICANT_BITS, significant_bits.len());

    // 1.significant * 2 ^ exponent - bias
    let mut result: f64 = 1.0;
    for (i, bit) in significant_bits.iter().enumerate() {
        if *bit {
            result += 2.0_f64.powi(-(i as i32) - 1);
        }
    }

    if result == 1.0 && exponent_with_bias == 0 {
        return 0.0;
    }

    result *= 2.0_f64.powi(exponent_with_bias - DOUBLE_BIAS);
    if sign { -result } else { result }
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
    use rusty_bit_vec::{MAX_INTEGER, MIN_INTEGER};

    use super::*;

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
    fn test_f64_to_bytes_ints() {
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
    fn test_f64_to_bytes_ints_str() {
        assert_eq!(
            fmt_double_bits(&f64_to_bits(0.0)),
            "0|00000000000|0000000000000000000000000000000000000000000000000000",
            "0"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(1.0)),
            "0|01111111111|0000000000000000000000000000000000000000000000000000",
            "1"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(-1.0)),
            "1|01111111111|0000000000000000000000000000000000000000000000000000",
            "-1"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(2.0)),
            "0|10000000000|0000000000000000000000000000000000000000000000000000",
            "2"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(3.0)),
            "0|10000000000|1000000000000000000000000000000000000000000000000000",
            "3"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(4.0)),
            "0|10000000001|0000000000000000000000000000000000000000000000000000",
            "4"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(5.0)),
            "0|10000000001|0100000000000000000000000000000000000000000000000000",
            "5"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(6.0)),
            "0|10000000001|1000000000000000000000000000000000000000000000000000",
            "6"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(7.0)),
            "0|10000000001|1100000000000000000000000000000000000000000000000000",
            "7"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(10.0)),
            "0|10000000010|0100000000000000000000000000000000000000000000000000",
            "10"
        );
    }

    #[test]
    fn test_f64_to_bytes_doubles_str() {
        assert_eq!(
            fmt_double_bits(&f64_to_bits(0.5)),
            "0|01111111110|0000000000000000000000000000000000000000000000000000",
            "0.5"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(-0.5)),
            "1|01111111110|0000000000000000000000000000000000000000000000000000",
            "-0.5"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(0.375)),
            "0|01111111101|1000000000000000000000000000000000000000000000000000",
            "0.375"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(0.25)),
            "0|01111111101|0000000000000000000000000000000000000000000000000000",
            "0.25"
        );
        assert_eq!(
            fmt_double_bits(&f64_to_bits(1.5)),
            "0|01111111111|1000000000000000000000000000000000000000000000000000",
            "1.5"
        );
    }

    #[test]
    fn test_bytes_to_f64_ints() {
        // let's try some integers
        for i in -100..=100 {
            let source: f64 = i as f64;
            let bytes = f64_to_bytes(source);
            let converted = bytes_to_f64(&bytes);
            assert_eq!(source, converted, "{}", source);
        }
    }

    #[test]
    fn test_bytes_to_f64_double() {
        for i in -100..=100 {
            let source: f64 = i as f64 * 0.1;
            let bytes = f64_to_bytes(source);
            let converted = bytes_to_f64(&bytes);
            assert_eq!(source, converted, "{}", source);
        }
    }
}
