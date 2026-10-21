use crate::{
    sequential_access::{SequentialReader, SequentialWriter},
    storage_errors::StorageError,
};
use std::path::PathBuf;

pub const RECORD_SIZE: u8 = 4;

#[derive(Debug)]
pub struct HeapStorage {}

impl HeapStorage {
    pub fn create_heap_file_path(storage_dir: &str) -> PathBuf {
        let mut file_path = PathBuf::from(storage_dir);
        file_path.push(HEAP_FILE_NAME);
        file_path
    }
}

impl SequentialWriter for HeapStorage {
    type ReaderType = HeapStorage;
    type RecordType = i32;

    fn get_record_size(&self) -> u8 {
        RECORD_SIZE
    }

    fn new() -> Self::ReaderType {
        HeapStorage {}
    }

    fn encode_record(&self, record: &Self::RecordType, slice: &mut [u8]) {
        slice.copy_from_slice(&record.to_le_bytes());
    }
}

impl SequentialReader for HeapStorage {
    type ReaderType = HeapStorage;
    type RecordType = i32;

    fn get_record_size(&self) -> u8 {
        RECORD_SIZE
    }

    fn new() -> Self::ReaderType {
        HeapStorage {}
    }

    fn decode_record(&self, buffer: &[u8]) -> Result<Self::RecordType, StorageError> {
        let decoded_record = i32::from_le_bytes(buffer.try_into().map_err(|_| {
            StorageError::HeapCorruptedRecord("Couldn't convert record to i32".to_string())
        })?);

        Ok(decoded_record)
    }
}

pub const HEAP_FILE_NAME: &'static str = "heap.dat";
