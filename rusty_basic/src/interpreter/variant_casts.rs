use crate::RuntimeError;
use rusty_linter::QBNumberCast;
use rusty_parser::FileHandle;
use rusty_variant::Variant;
use std::convert::TryFrom;

pub trait VariantCasts {
    fn to_file_handle(&self) -> Result<FileHandle, RuntimeError>;

    fn to_record_number(&self) -> Result<usize, RuntimeError>;

    fn to_non_negative_int(&self) -> Result<usize, RuntimeError>;

    fn to_positive_int(&self) -> Result<usize, RuntimeError>;

    fn to_positive_int_or(&self, err: RuntimeError) -> Result<usize, RuntimeError>;

    fn to_str_unchecked(&self) -> &str;
}

impl VariantCasts for Variant {
    fn to_file_handle(&self) -> Result<FileHandle, RuntimeError> {
        let i: i32 = self.try_cast()?;
        FileHandle::try_from(i).map_err(|_| RuntimeError::BadFileNameOrNumber)
    }

    fn to_record_number(&self) -> Result<usize, RuntimeError> {
        let record_number_as_long: i64 = self.try_cast()?;
        if record_number_as_long <= 0 {
            Err(RuntimeError::BadRecordNumber)
        } else {
            Ok(record_number_as_long as usize)
        }
    }

    fn to_non_negative_int(&self) -> Result<usize, RuntimeError> {
        let i: i32 = self.try_cast()?;
        if i >= 0 {
            Ok(i as usize)
        } else {
            Err(RuntimeError::IllegalFunctionCall)
        }
    }

    fn to_positive_int(&self) -> Result<usize, RuntimeError> {
        self.to_positive_int_or(RuntimeError::IllegalFunctionCall)
    }

    fn to_positive_int_or(&self, err: RuntimeError) -> Result<usize, RuntimeError> {
        let i: i32 = self.try_cast()?;
        if i > 0 {
            Ok(i as usize)
        } else {
            Err(err)
        }
    }

    /// Gets a `str` reference from this Variant.
    ///
    /// Panics if the variant is not of string type.
    ///
    /// Use it only at runtime if the linter has guaranteed the type.
    fn to_str_unchecked(&self) -> &str {
        match self {
            Variant::VString(s) => s,
            _ => panic!("Variant was not a string {:?}", self),
        }
    }
}
