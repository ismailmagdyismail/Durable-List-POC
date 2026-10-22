use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::storage_errors::StorageError;

#[derive(Debug)]
pub struct InPlaceStorageWriter {
    file: File,
}

impl InPlaceStorageWriter {
    pub fn open(file_path: PathBuf) -> Result<Self, StorageError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_path)
            .map_err(|error| StorageError::Open(error.to_string()))?;

        Ok(Self { file })
    }

    pub fn overwrite(&mut self, offset: u64, bytes: &[u8]) -> Result<(), StorageError> {
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|error| StorageError::Seek(error.to_string()))?;
        self.file
            .write_all(bytes)
            .map_err(|error| StorageError::Write(error.to_string()))?;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), StorageError> {
        self.file
            .sync_all()
            .map_err(|error| StorageError::Sync(error.to_string()))
    }
}
