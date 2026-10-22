use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::storage_errors::StorageError;

pub trait StorageEngine {
    fn write_node(&mut self, idx: u32, val: i32) -> Result<(), StorageError>;
}

pub struct MockLogFileStorageEngine {
    log_file_writer: BufWriter<File>,
}

impl MockLogFileStorageEngine {
    pub fn new(file_path: PathBuf) -> Result<Self, StorageError> {
        let mut options = OpenOptions::new();
        let log_file = options
            .create(true)
            .write(true)
            .open(file_path)
            .map_err(|err| StorageError::Open(err.to_string()))?;
        let log_file_writer = BufWriter::new(log_file);
        let engine = MockLogFileStorageEngine { log_file_writer };
        Ok(engine)
    }
}

impl StorageEngine for MockLogFileStorageEngine {
    fn write_node(&mut self, idx: u32, val: i32) -> Result<(), StorageError> {
        let mut buffer = Vec::new();
        let idx_str = idx.to_string();
        let val_str = val.to_string();
        buffer.resize(idx_str.len() + val_str.len(), b'0');
        buffer[0..idx_str.len()].copy_from_slice(idx_str.as_bytes());
        buffer[idx_str.len()..].copy_from_slice(val_str.as_bytes());

        self.log_file_writer
            .write_all(&buffer)
            .map_err(|err| StorageError::Write(err.to_string()))?;
        self.log_file_writer
            .write_all("\n\n".as_bytes())
            .map_err(|err| StorageError::Write(err.to_string()))?;

        self.log_file_writer
            .flush()
            .map_err(|err| StorageError::Flush(err.to_string()))?;
        self.log_file_writer
            .get_mut()
            .sync_all()
            .map_err(|err| StorageError::Sync(err.to_string()))?;

        Ok(())
    }
}
