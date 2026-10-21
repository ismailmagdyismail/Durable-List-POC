use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read},
};

use crate::{heap::create_heap_file_path, heap_storage::RECORD_SIZE, storage_errors::StorageError};

pub struct HeapCursor {
    buff_reader: BufReader<File>,
}

impl HeapCursor {
    pub fn new(storage_dir: &str) -> Result<HeapCursor, StorageError> {
        let heap_path = create_heap_file_path(storage_dir);
        let mut options = OpenOptions::new();
        let file = options.read(true).open(heap_path).map_err(|err| {
            return StorageError::HeadDataFile(err.to_string());
        })?;
        let buff_reader = BufReader::new(file);
        let cursor = HeapCursor { buff_reader };
        Ok(cursor)
    }

    pub fn next_record(&mut self) -> Result<Option<i32>, StorageError> {
        let mut buffer = Vec::new();
        buffer.resize(RECORD_SIZE as usize, b'0');

        let size = self
            .buff_reader
            .read(&mut buffer)
            .map_err(|err| StorageError::HeapRead(err.to_string()))?;

        if size == 0 {
            return Ok(None);
        }

        assert!(size == RECORD_SIZE as usize);

        let decoded_record = i32::from_le_bytes(buffer.try_into().map_err(|_| {
            StorageError::HeapCorruptedRecord("Couldn't convert record to i32".to_string())
        })?);

        Ok(Some(decoded_record))
    }
}
