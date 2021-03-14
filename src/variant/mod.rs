mod array_value;
mod bits;
mod casting;
mod fit;
mod user_defined_type_value;
mod variant;

pub use self::array_value::*;
pub use self::bits::*;
pub use self::user_defined_type_value::*;
pub use self::variant::*;

use crate::common::QError;

pub trait QBNumberCast<T> {
    fn try_cast(&self) -> Result<T, QError>;
}
