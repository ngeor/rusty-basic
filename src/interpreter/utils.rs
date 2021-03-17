use crate::common::{FileHandle, QError};
use crate::variant::{QBNumberCast, Variant};
use std::convert::TryFrom;

pub fn to_file_handle(v: &Variant) -> Result<FileHandle, QError> {
    let i: i32 = v.try_cast()?;
    FileHandle::try_from(i)
}

pub fn get_record_number(v: &Variant) -> Result<usize, QError> {
    let record_number_as_long: i64 = v.try_cast()?;
    if record_number_as_long <= 0 {
        Err(QError::BadRecordNumber)
    } else {
        Ok(record_number_as_long as usize)
    }
}
