use crate::common::{FileAccess, FileHandle, FileMode};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

#[derive(Debug)]
struct FileInfo {
    file: Option<File>,
    buf_reader: Option<BufReader<File>>,
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

    pub fn close(&mut self, handle: FileHandle) {
        self.handle_map.remove(&handle);
    }

    pub fn open(
        &mut self,
        handle: FileHandle,
        file_name: &str,
        file_mode: FileMode,
    ) -> std::io::Result<()> {
        if self.handle_map.contains_key(&handle) {
            return Err(std::io::Error::from(std::io::ErrorKind::AddrInUse));
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
                    },
                );
            }
        }
        Ok(())
    }

    pub fn print(&mut self, handle: FileHandle, print_args: Vec<String>) -> std::io::Result<()> {
        match self.handle_map.get_mut(&handle) {
            Some(file_info) => {
                for elem in print_args {
                    file_info.file.as_ref().unwrap().write(elem.as_bytes())?;
                }
                file_info.file.as_ref().unwrap().write(&[13, 10])?;
                Ok(())
            }
            None => Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
        }
    }

    pub fn read_line(&mut self, handle: FileHandle) -> std::io::Result<String> {
        match self.handle_map.get_mut(&handle) {
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

    pub fn eof(&mut self, handle: FileHandle) -> std::io::Result<bool> {
        match self.handle_map.get_mut(&handle) {
            Some(file_info) => {
                let buf = file_info.buf_reader.as_mut().unwrap().fill_buf()?;
                let len = buf.len();
                Ok(len == 0)
            }
            None => Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
        }
    }
}
