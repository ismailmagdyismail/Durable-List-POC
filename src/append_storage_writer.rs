use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::storage_errors::StorageError;

#[derive(Debug)]
pub struct AppendStorageWriter {
    writer: BufWriter<File>,
}

impl AppendStorageWriter {
    pub fn open(file_path: PathBuf) -> Result<Self, StorageError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .map_err(|error| StorageError::Open(error.to_string()))?;

        let mut writer = BufWriter::new(file);
        writer
            .seek(SeekFrom::End(0))
            .map_err(|error| StorageError::Seek(error.to_string()))?;

        Ok(Self { writer })
    }

    pub fn append(&mut self, bytes: &[u8]) -> Result<u64, StorageError> {
        let starting_offset = self
            .writer
            .stream_position()
            .map_err(|error| StorageError::Seek(error.to_string()))?;

        self.writer
            .write_all(bytes)
            .map_err(|error| StorageError::Write(error.to_string()))?;

        Ok(starting_offset)
    }

    pub fn flush(&mut self) -> Result<(), StorageError> {
        self.writer
            .flush()
            .map_err(|error| StorageError::Flush(error.to_string()))
    }

    pub fn sync(&mut self) -> Result<(), StorageError> {
        self.flush()?;
        self.writer
            .get_mut()
            .sync_all()
            .map_err(|error| StorageError::Sync(error.to_string()))
    }

    pub fn get_offset(&mut self) -> Result<u64, StorageError> {
        self.writer
            .stream_position()
            .map_err(|error| StorageError::Seek(error.to_string()))
    }
}
