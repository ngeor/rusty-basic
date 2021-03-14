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

// TODO get rid of the u8 here for all types (only the public api, internally it can still use u8 if needed)
// TODO move these outside the common module

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

pub const FILE_ACCESS_UNSPECIFIED: u8 = 1;
pub const FILE_ACCESS_READ: u8 = 2;
pub const FILE_ACCESS_WRITE: u8 = 3;

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
        if i == FILE_ACCESS_UNSPECIFIED {
            FileAccess::Unspecified
        } else if i == FILE_ACCESS_READ {
            FileAccess::Read
        } else if i == FILE_ACCESS_WRITE {
            FileAccess::Write
        } else {
            panic!("Unsupported file access {}", i)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_mode_u8_conversion() {
        let file_modes = [
            FileMode::Append,
            FileMode::Input,
            FileMode::Output,
            FileMode::Random,
        ];
        let numeric_values = [
            FILE_MODE_APPEND,
            FILE_MODE_INPUT,
            FILE_MODE_OUTPUT,
            FILE_MODE_RANDOM,
        ];
        assert_eq!(file_modes.len(), numeric_values.len());
        assert!(!file_modes.is_empty());
        for i in 0..file_modes.len() {
            assert_eq!(u8::from(file_modes[i]), numeric_values[i]);
            assert_eq!(FileMode::from(numeric_values[i]), file_modes[i]);
            assert!(numeric_values[i] > 0);
            if i > 0 {
                assert!(numeric_values[i] > numeric_values[i - 1]);
            }
        }
    }

    #[should_panic]
    #[test]
    fn test_zero_file_mode_should_panic() {
        FileMode::from(0);
    }

    #[test]
    fn test_file_access_u8_conversion() {
        let file_accesses = [FileAccess::Unspecified, FileAccess::Read, FileAccess::Write];
        let numeric_values = [FILE_ACCESS_UNSPECIFIED, FILE_ACCESS_READ, FILE_ACCESS_WRITE];
        assert_eq!(file_accesses.len(), numeric_values.len());
        assert!(!file_accesses.is_empty());
        for i in 0..file_accesses.len() {
            assert_eq!(u8::from(file_accesses[i]), numeric_values[i]);
            assert_eq!(FileAccess::from(numeric_values[i]), file_accesses[i]);
            assert!(numeric_values[i] > 0);
            if i > 0 {
                assert!(numeric_values[i] > numeric_values[i - 1]);
            }
        }
    }

    #[should_panic]
    #[test]
    fn test_zero_file_access_should_panic() {
        FileAccess::from(0);
    }
}
