use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read},
    path::PathBuf,
};

use crate::storage_errors::StorageError;

pub struct SequentialCursor {
    reader: BufReader<File>,
}

impl SequentialCursor {
    pub fn open(file_path: PathBuf) -> Result<Self, StorageError> {
        let file = OpenOptions::new()
            .read(true)
            .open(file_path)
            .map_err(|error| StorageError::Open(error.to_string()))?;

        Ok(Self {
            reader: BufReader::new(file),
        })
    }

    pub fn next_record(&mut self, record_size: usize) -> Result<Option<Vec<u8>>, StorageError> {
        if record_size == 0 {
            return Err(StorageError::InvalidRecordSize);
        }

        let mut bytes = vec![0; record_size];
        let mut bytes_read = 0;

        while bytes_read < record_size {
            match self.reader.read(&mut bytes[bytes_read..]) {
                Ok(0) if bytes_read == 0 => return Ok(None),
                Ok(0) => {
                    return Err(StorageError::TruncatedRecord {
                        expected: record_size,
                        actual: bytes_read,
                    });
                }
                Ok(read) => bytes_read += read,
                Err(error) => return Err(StorageError::Read(error.to_string())),
            }
        }

        Ok(Some(bytes))
    }
}
