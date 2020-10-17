use crate::common::QError;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FileMode {
    Input,
    Output,
    Append,
}

impl From<FileMode> for i32 {
    fn from(f: FileMode) -> i32 {
        match f {
            FileMode::Input => 1,
            FileMode::Output => 2,
            FileMode::Append => 3,
        }
    }
}

impl From<i32> for FileMode {
    fn from(i: i32) -> FileMode {
        if i == 1 {
            FileMode::Input
        } else if i == 2 {
            FileMode::Output
        } else if i == 3 {
            FileMode::Append
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

impl From<FileAccess> for i32 {
    fn from(f: FileAccess) -> i32 {
        match f {
            FileAccess::Unspecified => 0,
            FileAccess::Read => 1,
            FileAccess::Write => 2,
        }
    }
}

impl From<i32> for FileAccess {
    fn from(i: i32) -> FileAccess {
        if i == 1 {
            FileAccess::Read
        } else if i == 2 {
            FileAccess::Write
        } else {
            FileAccess::Unspecified
        }
    }
}

#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

impl From<FileHandle> for u8 {
    fn from(file_handle: FileHandle) -> u8 {
        file_handle.0
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

impl std::fmt::Debug for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl std::fmt::Display for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
