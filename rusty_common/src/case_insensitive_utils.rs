use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Creates a hash for a [str] ignoring case.
pub fn hash_str<H: Hasher>(s: &str, state: &mut H) {
    for byte in s.as_bytes() {
        byte.to_ascii_uppercase().hash(state);
    }
}

/// Compares two [str] ignoring case.
pub fn cmp_str(left: &str, right: &str) -> Ordering {
    cmp_bytes(left.as_bytes(), right.as_bytes())
}

fn cmp_bytes(left: &[u8], right: &[u8]) -> Ordering {
    let mut i = 0;
    while i < left.len() && i < right.len() {
        let ord = left[i]
            .to_ascii_uppercase()
            .cmp(&right[i].to_ascii_uppercase());
        if ord != Ordering::Equal {
            return ord;
        }
        i += 1;
    }

    if i < right.len() {
        Ordering::Less
    } else if i < left.len() {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}
