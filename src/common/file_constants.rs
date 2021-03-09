use crate::common::QError;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FileMode {
    Append,
    Input,
    Output,
    Random,
}

pub const FILE_MODE_APPEND: u8 = 1;
pub const FILE_MODE_INPUT: u8 = 2;
pub const FILE_MODE_OUTPUT: u8 = 3;
pub const FILE_MODE_RANDOM: u8 = 4;

impl From<FileMode> for u8 {
    fn from(f: FileMode) -> u8 {
        match f {
            FileMode::Append => FILE_MODE_APPEND,
            FileMode::Input => FILE_MODE_INPUT,
            FileMode::Output => FILE_MODE_OUTPUT,
            FileMode::Random => FILE_MODE_RANDOM,
        }
    }
}

impl From<u8> for FileMode {
    fn from(i: u8) -> FileMode {
        if i == FILE_MODE_APPEND {
            FileMode::Append
        } else if i == FILE_MODE_INPUT {
            FileMode::Input
        } else if i == FILE_MODE_OUTPUT {
            FileMode::Output
        } else if i == FILE_MODE_RANDOM {
            FileMode::Random
        } else {
            panic!("Unsupported file mode {}", i)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FileAccess {
    Unspecified,
    Read,
    Write,
}

pub const FILE_ACCESS_UNSPECIFIED: u8 = 0;
pub const FILE_ACCESS_READ: u8 = 1;
pub const FILE_ACCESS_WRITE: u8 = 2;

impl From<FileAccess> for u8 {
    fn from(f: FileAccess) -> u8 {
        match f {
            FileAccess::Unspecified => FILE_ACCESS_UNSPECIFIED,
            FileAccess::Read => FILE_ACCESS_READ,
            FileAccess::Write => FILE_ACCESS_WRITE,
        }
    }
}

impl From<u8> for FileAccess {
    fn from(i: u8) -> FileAccess {
        if i == FILE_ACCESS_READ {
            FileAccess::Read
        } else if i == FILE_ACCESS_WRITE {
            FileAccess::Write
        } else {
            FileAccess::Unspecified
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileHandle(u8);

impl FileHandle {
    pub fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

impl From<u8> for FileHandle {
    fn from(x: u8) -> FileHandle {
        FileHandle(x)
    }
}

impl From<FileHandle> for i32 {
    fn from(file_handle: FileHandle) -> i32 {
        file_handle.0 as i32
    }
}

impl TryFrom<i32> for FileHandle {
    type Error = QError;

    fn try_from(i: i32) -> Result<Self, Self::Error> {
        if i >= 1 && i <= 255 {
            Ok((i as u8).into())
        } else {
            Err(QError::BadFileNameOrNumber)
        }
    }
}
