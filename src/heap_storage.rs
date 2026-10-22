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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_heap_path_in_storage_directory() {
        assert_eq!(
            HeapStorage::create_heap_file_path("storage"),
            PathBuf::from("storage").join(HEAP_FILE_NAME)
        );
    }

    #[test]
    fn reports_four_byte_records_for_reading_and_writing() {
        let writer = <HeapStorage as SequentialWriter>::new();
        let reader = <HeapStorage as SequentialReader>::new();

        assert_eq!(SequentialWriter::get_record_size(&writer), RECORD_SIZE);
        assert_eq!(SequentialReader::get_record_size(&reader), RECORD_SIZE);
    }

    #[test]
    fn encodes_and_decodes_i32_values_as_little_endian() {
        let writer = <HeapStorage as SequentialWriter>::new();
        let reader = <HeapStorage as SequentialReader>::new();

        for record in [0, 1, -1, i32::MIN, i32::MAX] {
            let mut buffer = [0; RECORD_SIZE as usize];
            writer.encode_record(&record, &mut buffer);

            assert_eq!(buffer, record.to_le_bytes());
            assert_eq!(reader.decode_record(&buffer).unwrap(), record);
        }
    }

    #[test]
    fn rejects_records_that_are_not_four_bytes() {
        let reader = <HeapStorage as SequentialReader>::new();

        let error = reader.decode_record(&[0; 3]).unwrap_err();

        assert!(matches!(error, StorageError::HeapCorruptedRecord(_)));
    }
}
