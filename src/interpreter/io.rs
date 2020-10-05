use crate::common::{FileAccess, FileHandle, FileMode, QError};
use crate::interpreter::Printer;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

#[derive(Debug)]
pub struct FileInfo {
    file: Option<File>,
    buf_reader: Option<BufReader<File>>,
    last_print_col: usize,
}

impl Printer for FileInfo {
    fn print(&mut self, s: String) -> std::io::Result<usize> {
        self.file.as_ref().unwrap().write(s.as_bytes())
    }

    fn get_last_print_col(&self) -> usize {
        self.last_print_col
    }

    fn set_last_print_col(&mut self, col: usize) {
        self.last_print_col = col;
    }
}

#[derive(Debug)]
pub struct FileManager {
    handle_map: HashMap<FileHandle, FileInfo>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            handle_map: HashMap::new(),
        }
    }

    pub fn close(&mut self, handle: &FileHandle) {
        self.handle_map.remove(handle);
    }

    pub fn close_all(&mut self) {
        self.handle_map.clear();
    }

    pub fn open(
        &mut self,
        handle: FileHandle,
        file_name: &str,
        file_mode: FileMode,
        _file_access: FileAccess,
    ) -> Result<(), QError> {
        if self.handle_map.contains_key(&handle) {
            return Err(QError::FileAlreadyOpen);
        }

        match file_mode {
            FileMode::Input => {
                let file = File::open(file_name)?;
                let buf_reader = BufReader::new(file);
                self.handle_map.insert(
                    handle,
                    FileInfo {
                        file: None,
                        buf_reader: Some(buf_reader),
                        last_print_col: 0,
                    },
                );
            }
            FileMode::Output => {
                let file = File::create(file_name)?;
                self.handle_map.insert(
                    handle,
                    FileInfo {
                        file: Some(file),
                        buf_reader: None,
                        last_print_col: 0,
                    },
                );
            }
            FileMode::Append => {
                let file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(file_name)?;
                self.handle_map.insert(
                    handle,
                    FileInfo {
                        file: Some(file),
                        buf_reader: None,
                        last_print_col: 0,
                    },
                );
            }
        }
        Ok(())
    }

    pub fn get_file_info_mut(&mut self, handle: &FileHandle) -> Option<&mut FileInfo> {
        self.handle_map.get_mut(handle)
    }

    pub fn read_line(&mut self, handle: &FileHandle) -> std::io::Result<String> {
        match self.handle_map.get_mut(handle) {
            Some(file_info) => {
                let mut buf = String::new();
                file_info.buf_reader.as_mut().unwrap().read_line(&mut buf)?;
                let cr_lf: &[_] = &['\r', '\n'];
                buf = buf.trim_end_matches(cr_lf).to_string();
                Ok(buf)
            }
            None => Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
        }
    }

    pub fn eof(&mut self, handle: &FileHandle) -> std::io::Result<bool> {
        match self.handle_map.get_mut(handle) {
            Some(file_info) => {
                let buf = file_info.buf_reader.as_mut().unwrap().fill_buf()?;
                let len = buf.len();
                Ok(len == 0)
            }
            None => Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
        }
    }
}
