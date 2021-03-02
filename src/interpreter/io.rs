use crate::common::{FileAccess, FileHandle, FileMode, QError};
use crate::interpreter::read_input::ReadInputSource;
use crate::interpreter::write_printer::WritePrinter;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::BufReader;

pub enum FileInfo {
    Input(FileInfoInput),
    Output(FileInfoOutput),
}

pub type FileInfoInput = ReadInputSource<BufReader<File>>;
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
                self.handle_map.insert(
                    handle,
                    FileInfo::Input(FileInfoInput::new(BufReader::new(file))),
                );
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

    fn try_get_file_info(&self, handle: &FileHandle) -> Result<&FileInfo, QError> {
        match self.handle_map.get(handle) {
            Some(f) => Ok(f),
            None => Err(QError::FileNotFound),
        }
    }

    pub fn try_get_file_info_input(&self, handle: &FileHandle) -> Result<&FileInfoInput, QError> {
        match self.try_get_file_info(handle) {
            Ok(FileInfo::Input(input)) => Ok(input),
            Ok(FileInfo::Output(_)) => Err(QError::BadFileMode),
            Err(err) => Err(err),
        }
    }

    pub fn try_get_file_info_output(&self, handle: &FileHandle) -> Result<&FileInfoOutput, QError> {
        match self.try_get_file_info(handle) {
            Ok(FileInfo::Output(output)) => Ok(output),
            Ok(FileInfo::Input(_)) => Err(QError::BadFileMode),
            Err(err) => Err(err),
        }
    }
}
