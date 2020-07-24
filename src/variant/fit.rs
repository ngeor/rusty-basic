use super::{Variant, MAX_INTEGER, MAX_LONG, MIN_INTEGER, MIN_LONG};

trait IsInRange {
    fn is_in_integer_range(self) -> bool;
    fn is_in_long_range(self) -> bool;
}

impl IsInRange for i32 {
    fn is_in_integer_range(self) -> bool {
        self >= MIN_INTEGER && self <= MAX_INTEGER
    }

    fn is_in_long_range(self) -> bool {
        (self as i64).is_in_long_range()
    }
}

impl IsInRange for i64 {
    fn is_in_integer_range(self) -> bool {
        self >= (MIN_INTEGER as i64) && self <= (MAX_INTEGER as i64)
    }

    fn is_in_long_range(self) -> bool {
        self >= MIN_LONG && self <= MAX_LONG
    }
}

pub trait FitToType {
    fn fit_to_type(self) -> Variant;
}

impl FitToType for i32 {
    fn fit_to_type(self) -> Variant {
        if self.is_in_integer_range() {
            Variant::VInteger(self)
        } else if self.is_in_long_range() {
            Variant::VLong(self as i64)
        } else {
            Variant::VDouble(self as f64)
        }
    }
}

impl FitToType for i64 {
    fn fit_to_type(self) -> Variant {
        if self.is_in_integer_range() {
            Variant::VInteger(self as i32)
        } else if self.is_in_long_range() {
            Variant::VLong(self)
        } else {
            Variant::VDouble(self as f64)
        }
    }
}

impl FitToType for f32 {
    fn fit_to_type(self) -> Variant {
        let diff = self - self.round();
        let has_fraction = diff.abs() > 0.0001;
        if has_fraction {
            Variant::VSingle(self)
        } else {
            (self.round() as i64).fit_to_type()
        }
    }
}

impl FitToType for f64 {
    fn fit_to_type(self) -> Variant {
        let diff = self - self.round();
        let has_fraction = diff.abs() > 0.0001;
        if has_fraction {
            Variant::VDouble(self)
        } else {
            (self.round() as i64).fit_to_type()
        }
    }
}
