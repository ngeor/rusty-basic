use crate::common::{FileAccess, FileHandle, FileMode, QError};
use crate::interpreter::input_source::InputSource;
use crate::interpreter::printer::WritePrinter;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read};

pub enum FileInfo {
    Input(FileInfoInput),
    Output(FileInfoOutput),
}

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

    pub fn eof(&mut self) -> Result<bool, QError> {
        if self.previous_buffer.is_empty() {
            let buf = self.buf_reader.fill_buf()?;
            let len = buf.len();
            Ok(len == 0)
        } else {
            Ok(false)
        }
    }
}

// TODO same implementation for stdio
// TODO for MockStdlib only the stdio source should be overriden
impl InputSource for FileInfoInput {
    fn input(&mut self) -> std::io::Result<String> {
        read_until_comma_or_eol(&mut self.buf_reader, &mut self.previous_buffer)
    }

    fn line_input(&mut self) -> std::io::Result<String> {
        let mut buf = String::new();
        self.buf_reader.read_line(&mut buf)?;
        let cr_lf: &[_] = &['\r', '\n'];
        buf = buf.trim_end_matches(cr_lf).to_string();
        Ok(buf)
    }
}

pub type FileInfoOutput = WritePrinter<File>;

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

struct StringRefReader<'a> {
    buf: &'a mut String,
}

impl<'a> Read for StringRefReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut i: usize = 0;
        while i < buf.len() && !self.buf.is_empty() {
            buf[i] = self.buf.remove(0) as u8;
            i += 1;
        }
        Ok(i)
    }
}

impl<'a> BufRead for StringRefReader<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(self.buf.as_bytes())
    }

    fn consume(&mut self, amt: usize) {
        for _i in 0..amt {
            self.buf.remove(0);
        }
    }
}

pub fn read_until_comma_or_eol<T: BufRead>(
    buf_read: &mut T,
    previous_buffer: &mut String,
) -> std::io::Result<String> {
    // create a temporary reader that reads first from previous_buffer and then from buf_read if needed
    let str_ref_reader = StringRefReader {
        buf: previous_buffer,
    };
    let mut chain = str_ref_reader.chain(buf_read);

    let mut buf: String = String::new();
    let bytes_read = chain.read_line(&mut buf)?;
    let mut remainder: String = String::new();
    if bytes_read == 0 {
        // EOF
    } else {
        match buf.find(',') {
            Some(comma_pos) => {
                remainder = buf.split_off(comma_pos);
                remainder = remainder.split_off(1);
            }
            None => {}
        }
    }

    buf = buf.trim().to_string();
    previous_buffer.insert_str(0, remainder.as_str());
    Ok(buf)
}
