use crate::common::{FileAccess, FileHandle, FileMode, QError};
use crate::interpreter::read_input::ReadInputSource;
use crate::interpreter::write_printer::WritePrinter;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

pub type FileInfoInput = ReadInputSource<BufReader<File>>;
pub type FileInfoOutput = WritePrinter<File>;

pub struct FileInfo {
    field_lists: Vec<Vec<Field>>,
    input: Option<FileInfoInput>,
    output: Option<FileInfoOutput>,
    random: Option<File>,
    rec_len: usize,
    current_field_list_index: Option<usize>,
}

impl FileInfo {
    pub fn new_input(file: File) -> Self {
        Self {
            field_lists: vec![],
            input: Some(ReadInputSource::new(BufReader::new(file))),
            output: None,
            random: None,
            rec_len: 0,
            current_field_list_index: None,
        }
    }

    pub fn new_output(file: File) -> Self {
        Self {
            field_lists: vec![],
            input: None,
            output: Some(WritePrinter::new(file)),
            random: None,
            rec_len: 0,
            current_field_list_index: None,
        }
    }

    pub fn new_random(file: File, rec_len: usize) -> Self {
        Self {
            field_lists: vec![],
            input: None,
            output: None,
            random: Some(file),
            rec_len,
            current_field_list_index: None,
        }
    }

    pub fn add_field_list(&mut self, fields: Vec<Field>) {
        self.current_field_list_index = Some(self.field_lists.len());
        self.field_lists.push(fields);
    }

    pub fn get_field_lists(&self) -> &Vec<Vec<Field>> {
        &self.field_lists
    }

    pub fn get_current_field_list(&self) -> Option<&Vec<Field>> {
        match self.current_field_list_index {
            Some(idx) => self.field_lists.get(idx),
            _ => None,
        }
    }

    pub fn get_record(&mut self, record_number: usize) -> Result<Vec<u8>, QError> {
        self.ensure_random()?;
        if record_number <= 0 {
            return Err(QError::BadRecordNumber);
        }
        let offset = ((record_number - 1) * self.rec_len) as u64;
        let file = self.random.as_mut().expect("Should have file");
        file.seek(SeekFrom::Start(offset))?;
        let mut buffer: Vec<u8> = std::iter::repeat(0 as u8).take(self.rec_len).collect();
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read < buffer.len() {
            Err(QError::InputPastEndOfFile)
        } else {
            Ok(buffer)
        }
    }

    pub fn put_record(&mut self, record_number: usize, bytes: &[u8]) -> Result<(), QError> {
        self.ensure_random()?;
        if record_number <= 0 {
            return Err(QError::BadRecordNumber);
        }
        let offset = ((record_number - 1) * self.rec_len) as u64;
        let file = self.random.as_mut().expect("Should have file");
        file.seek(SeekFrom::Start(offset))?;
        file.write(bytes)?;
        Ok(())
    }

    fn ensure_random(&self) -> Result<(), QError> {
        if self.random.is_none() {
            Err(QError::BadFileMode)
        } else if self.rec_len <= 0 {
            Err(QError::BadRecordLength)
        } else {
            Ok(())
        }
    }
}

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
        rec_len: usize,
    ) -> Result<(), QError> {
        if self.handle_map.contains_key(&handle) {
            return Err(QError::FileAlreadyOpen);
        }

        match file_mode {
            FileMode::Random => {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(file_name)?;
                self.handle_map
                    .insert(handle, FileInfo::new_random(file, rec_len));
            }
            FileMode::Input => {
                let file = File::open(file_name)?;
                self.handle_map.insert(handle, FileInfo::new_input(file));
            }
            FileMode::Output => {
                let file = File::create(file_name)?;
                self.handle_map.insert(handle, FileInfo::new_output(file));
            }
            FileMode::Append => {
                let file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(file_name)?;
                self.handle_map.insert(handle, FileInfo::new_output(file));
            }
        }
        Ok(())
    }

    pub fn try_get_file_info(&self, handle: &FileHandle) -> Result<&FileInfo, QError> {
        self.handle_map.get(handle).ok_or(QError::FileNotFound)
    }

    pub fn try_get_file_info_mut(&mut self, handle: &FileHandle) -> Result<&mut FileInfo, QError> {
        self.handle_map.get_mut(handle).ok_or(QError::FileNotFound)
    }

    pub fn try_get_file_info_input(&self, handle: &FileHandle) -> Result<&FileInfoInput, QError> {
        let file_info = self.try_get_file_info(handle)?;
        file_info.input.as_ref().ok_or(QError::BadFileMode)
    }

    pub fn try_get_file_info_output(&self, handle: &FileHandle) -> Result<&FileInfoOutput, QError> {
        let file_info = self.try_get_file_info(handle)?;
        file_info.output.as_ref().ok_or(QError::BadFileMode)
    }

    pub fn add_field_list(&mut self, handle: FileHandle, fields: Vec<Field>) -> Result<(), QError> {
        // TODO if sum(field width) > rec_len, throw error
        let file_info = self.try_get_file_info_mut(&handle)?;
        file_info.add_field_list(fields);
        Ok(())
    }

    pub fn mark_current_field_list(&mut self, variable_name: &str) -> Result<(), QError> {
        for file_info in self.handle_map.values_mut() {
            for i in 0..file_info.field_lists.len() {
                let field_list = &file_info.field_lists[i];
                for field in field_list {
                    if field.name == variable_name {
                        // found it
                        file_info.current_field_list_index = Some(i);
                        return Ok(());
                    }
                }
            }
        }
        Err(QError::Other(format!(
            "Variable {} not used in FIELD",
            variable_name
        )))
    }
}

#[derive(Clone)]
pub struct Field {
    pub width: usize,
    pub name: String,
}
