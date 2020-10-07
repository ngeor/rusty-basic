use crate::common::{FileAccess, FileHandle, FileMode, QError};
use crate::interpreter::Printer;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Cursor, Read, Write};

#[derive(Debug)]
pub enum FileInfo {
    Input(FileInfoInput),
    Output(FileInfoOutput),
}

#[derive(Debug)]
pub struct FileInfoInput {
    buf_reader: BufReader<File>,
    previous_buffer: String,
}

impl FileInfoInput {
    pub fn new(file: File) -> Self {
        Self {
            buf_reader: BufReader::new(file),
            previous_buffer: String::new(),
        }
    }

    pub fn read_line(&mut self) -> Result<String, QError> {
        let mut buf = String::new();
        self.buf_reader.read_line(&mut buf)?;
        let cr_lf: &[_] = &['\r', '\n'];
        buf = buf.trim_end_matches(cr_lf).to_string();
        Ok(buf)
    }

    pub fn read_until_comma_or_eol(&mut self) -> Result<String, QError> {
        // read_until_comma_or_eol(file_input.buf_reader, file_input.previous_buffer)
        let mut buf = String::new();
        self.buf_reader.read_line(&mut buf)?;
        let cr_lf: &[_] = &['\r', '\n'];
        buf = buf.trim_end_matches(cr_lf).to_string();
        Ok(buf)
    }

    pub fn eof(&mut self) -> Result<bool, QError> {
        // TODO take previous buffer into account
        let buf = self.buf_reader.fill_buf()?;
        let len = buf.len();
        Ok(len == 0)
    }
}

#[derive(Debug)]
pub struct FileInfoOutput {
    file: File,
    last_print_col: usize,
}

impl FileInfoOutput {
    pub fn new(file: File) -> Self {
        Self {
            file,
            last_print_col: 0,
        }
    }
}

impl Printer for FileInfoOutput {
    fn print(&mut self, s: String) -> std::io::Result<usize> {
        self.file.write(s.as_bytes())
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
                self.handle_map
                    .insert(handle, FileInfo::Input(FileInfoInput::new(file)));
            }
            FileMode::Output => {
                let file = File::create(file_name)?;
                self.handle_map
                    .insert(handle, FileInfo::Output(FileInfoOutput::new(file)));
            }
            FileMode::Append => {
                let file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(file_name)?;
                self.handle_map
                    .insert(handle, FileInfo::Output(FileInfoOutput::new(file)));
            }
        }
        Ok(())
    }

    fn try_get_file_info_mut(&mut self, handle: &FileHandle) -> Result<&mut FileInfo, QError> {
        match self.handle_map.get_mut(handle) {
            Some(f) => Ok(f),
            None => Err(QError::FileNotFound),
        }
    }

    pub fn try_get_file_info_input_mut(
        &mut self,
        handle: &FileHandle,
    ) -> Result<&mut FileInfoInput, QError> {
        match self.try_get_file_info_mut(handle) {
            Ok(FileInfo::Input(input)) => Ok(input),
            Ok(FileInfo::Output(_)) => Err(QError::BadFileMode),
            Err(err) => Err(err),
        }
    }

    pub fn try_get_file_info_output_mut(
        &mut self,
        handle: &FileHandle,
    ) -> Result<&mut FileInfoOutput, QError> {
        match self.try_get_file_info_mut(handle) {
            Ok(FileInfo::Output(output)) => Ok(output),
            Ok(FileInfo::Input(_)) => Err(QError::BadFileMode),
            Err(err) => Err(err),
        }
    }
}

pub fn read_until_comma_or_eol<T: BufRead>(
    buf_read: T,
    previous_buffer: String,
) -> std::io::Result<(T, String, String)> {
    // create a temporary reader that reads first from previous_buffer and then from buf_read if needed
    let mut chain = Cursor::new(previous_buffer.into_bytes()).chain(buf_read);

    let mut buf: String = String::new();
    let bytes_read = chain.read_line(&mut buf)?;
    let mut remainder: String = String::new();
    if bytes_read == 0 {
        // EOF
    } else {
        match buf.find(',') {
            Some(comma_pos) => {
                remainder = buf.split_off(comma_pos + 1);
            }
            None => {
                // TODO trim trailing CR LF
            }
        }
    }

    let (cursor, buf_read) = chain.into_inner();
    let bytes = cursor.into_inner();
    let mut previous_buffer: String = String::from_utf8(bytes).unwrap();
    previous_buffer.insert_str(0, remainder.as_str());
    Ok((buf_read, previous_buffer, buf))
}
