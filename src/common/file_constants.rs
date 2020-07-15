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

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileHandle(u32);

impl FileHandle {
    pub fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

impl From<u32> for FileHandle {
    fn from(x: u32) -> FileHandle {
        FileHandle(x)
    }
}
