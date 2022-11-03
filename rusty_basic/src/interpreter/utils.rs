use rusty_common::{FileHandle, QError};
use rusty_linter::QBNumberCast;
use rusty_variant::Variant;
use std::convert::TryFrom;

pub trait VariantCasts {
    fn to_file_handle(&self) -> Result<FileHandle, QError>;

    fn to_record_number(&self) -> Result<usize, QError>;

    fn to_non_negative_int(&self) -> Result<usize, QError>;

    fn to_positive_int(&self) -> Result<usize, QError>;

    fn to_positive_int_or(&self, err: QError) -> Result<usize, QError>;
}

impl VariantCasts for Variant {
    fn to_file_handle(&self) -> Result<FileHandle, QError> {
        let i: i32 = self.try_cast()?;
        FileHandle::try_from(i)
    }

    fn to_record_number(&self) -> Result<usize, QError> {
        let record_number_as_long: i64 = self.try_cast()?;
        if record_number_as_long <= 0 {
            Err(QError::BadRecordNumber)
        } else {
            Ok(record_number_as_long as usize)
        }
    }

    fn to_non_negative_int(&self) -> Result<usize, QError> {
        let i: i32 = self.try_cast()?;
        if i >= 0 {
            Ok(i as usize)
        } else {
            Err(QError::IllegalFunctionCall)
        }
    }

    fn to_positive_int(&self) -> Result<usize, QError> {
        self.to_positive_int_or(QError::IllegalFunctionCall)
    }

    fn to_positive_int_or(&self, err: QError) -> Result<usize, QError> {
        let i: i32 = self.try_cast()?;
        if i > 0 {
            Ok(i as usize)
        } else {
            Err(err)
        }
    }
}
